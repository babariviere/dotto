//! Implements one way synchronisation

use crate::error::Result;
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, DirEntry, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum DiffKind {
    Added,
    // only for file
    Modified,
    Deleted,
}

impl fmt::Display for DiffKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiffKind::Added => write!(f, "add"),
            DiffKind::Modified => write!(f, "mod"),
            DiffKind::Deleted => write!(f, "del"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Diff {
    path: PathBuf,
    kind: DiffKind,
}

impl Diff {
    pub fn new<P: Into<PathBuf>>(path: P, kind: DiffKind) -> Diff {
        Diff {
            path: path.into(),
            kind,
        }
    }

    // Returns a relative path
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn kind(&self) -> &DiffKind {
        &self.kind
    }
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.path.parent().is_some() {
            return write!(f, "{} {}", self.kind, self.path.display());
        }
        write!(f, "{} <root>", self.kind)
    }
}

#[derive(Debug)]
enum FileType {
    Dir,
    File,
    None,
}

impl FileType {
    fn new<P: AsRef<Path>>(p: P) -> FileType {
        let p = p.as_ref();
        if !p.exists() {
            return FileType::None;
        }
        match p.is_dir() {
            true => FileType::Dir,
            false => FileType::File,
        }
    }

    fn exists(&self) -> bool {
        match self {
            FileType::None => false,
            _ => true,
        }
    }
}

fn checksum<A, B>(src: A, dst: B) -> Result<bool>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let mut fsrc = File::open(src)?;
    let mut fdst = File::open(dst)?;
    let mut bsrc = Vec::new();
    let mut bdst = Vec::new();
    fsrc.read_to_end(&mut bsrc)?;
    fdst.read_to_end(&mut bdst)?;

    let sumsrc = md5::compute(bsrc);
    let sumdst = md5::compute(bdst);

    Ok(sumsrc == sumdst)
}

fn sync_diff_rec<A, B, C>(src_root: A, dst_root: B, file: C) -> Result<Vec<Diff>>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
    C: AsRef<Path>,
{
    let mut diffs = Vec::new();
    let file: &Path = file.as_ref();
    let src_root = src_root.as_ref();
    let dst_root = dst_root.as_ref();
    let mut src = src_root.to_owned();
    let mut dst = dst_root.to_owned();
    if file.parent().is_some() {
        src = src.join(file);
        dst = dst.join(file);
    }
    let src_ty = FileType::new(&src);
    let dst_ty = FileType::new(&dst);

    match (&src_ty, &dst_ty) {
        (FileType::File, FileType::File) => {
            if !checksum(&src, &dst)? {
                diffs.push(Diff::new(file, DiffKind::Modified));
            }
        }
        (FileType::File, FileType::None) => {
            diffs.push(Diff::new(file, DiffKind::Added));
        }
        (FileType::None, FileType::File) => {
            diffs.push(Diff::new(file, DiffKind::Deleted));
        }
        (FileType::Dir, FileType::File) | (FileType::Dir, FileType::None) => {
            if dst_ty.exists() {
                diffs.push(Diff::new(file, DiffKind::Deleted));
            }
            diffs.push(Diff::new(file, DiffKind::Added));
            for entry in src.read_dir()? {
                let entry: DirEntry = entry?;
                let entry_diffs =
                    sync_diff_rec(&src_root, &dst_root, file.join(entry.file_name()))?;
                diffs.extend(entry_diffs.into_iter());
            }
        }
        (FileType::File, FileType::Dir) | (FileType::None, FileType::Dir) => {
            for entry in dst.read_dir()? {
                let entry: DirEntry = entry?;
                let entry_diffs =
                    sync_diff_rec(&src_root, &dst_root, file.join(entry.file_name()))?;
                diffs.extend(entry_diffs.into_iter());
            }
            diffs.push(Diff::new(file, DiffKind::Deleted));
            if src_ty.exists() {
                diffs.push(Diff::new(file, DiffKind::Added));
            }
        }
        (FileType::Dir, FileType::Dir) => {
            let mut hash_set = HashSet::new();
            src.read_dir()?
                .map(|f| f.map(|f| hash_set.insert(file.join(f.file_name()))))
                .collect::<std::io::Result<Vec<_>>>()?;
            dst.read_dir()?
                .map(|f| f.map(|f| hash_set.insert(file.join(f.file_name()))))
                .collect::<std::io::Result<Vec<_>>>()?;
            for file in hash_set {
                let entry_diffs = sync_diff_rec(&src_root, &dst_root, file)?;
                diffs.extend(entry_diffs.into_iter());
            }
        }
        (FileType::None, FileType::None) => {}
    }

    Ok(diffs)
}

// Compute diff between two folders
// Returned path will be relative
pub fn sync_diff<A, B>(src: A, dst: B) -> Result<Vec<Diff>>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let src: &Path = src.as_ref();
    let dst: &Path = dst.as_ref();
    if !src.exists() {
        return Err(io::Error::from(io::ErrorKind::NotFound).into());
    }
    return sync_diff_rec(src, dst, "");
}

// One way sync from src to dst
pub fn sync<A, B>(src: A, dst: B, diffs: &[Diff]) -> Result<()>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let src = src.as_ref();
    let dst = dst.as_ref();
    for diff in diffs {
        let src_path = src.join(diff.path());
        let dst_path = dst.join(diff.path());
        match diff.kind() {
            DiffKind::Modified => {
                fs::copy(src_path, dst_path)?;
            }
            DiffKind::Added => {
                if src_path.is_dir() {
                    fs::create_dir(dst_path)?;
                } else {
                    fs::copy(src_path, dst_path)?;
                }
            }
            DiffKind::Deleted => {
                if dst_path.is_dir() {
                    fs::remove_dir(dst_path)?;
                } else {
                    fs::remove_file(dst_path)?;
                }
            }
        }
    }
    return Ok(());
}
