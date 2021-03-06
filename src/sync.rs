//! Implements one way synchronisation

use crate::error::{DotError, Result};
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

#[derive(Debug)]
pub struct SyncSettings {
    pub depth: usize,
    pub recursive: bool,
    pub exclude: Vec<glob::Pattern>,
}

impl SyncSettings {
    // TODO: String -> AsRef<str>
    pub fn new(mut depth: usize, recursive: bool, exclude: &[String]) -> Result<SyncSettings> {
        if depth == 0 {
            depth = 1;
        }
        Ok(SyncSettings {
            depth,
            recursive,
            exclude: exclude
                .iter()
                .map(|s| glob::Pattern::new(s).map_err(|e| DotError::wrap(s, e)))
                .collect::<std::result::Result<Vec<glob::Pattern>, _>>()?,
        })
    }
}

#[derive(Clone, Debug)]
struct SyncContext<'a> {
    current_depth: usize,
    settings: &'a SyncSettings,
}

impl SyncContext<'_> {
    fn deeper(&self) -> SyncContext<'_> {
        SyncContext {
            current_depth: self.current_depth + 1,
            settings: self.settings,
        }
    }

    fn too_deep(&self) -> bool {
        self.current_depth >= self.settings.depth && !self.settings.recursive
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

fn sync_diff_rec<A, B, C>(ctx: SyncContext, src_root: A, dst_root: B, file: C) -> Result<Vec<Diff>>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
    C: AsRef<Path>,
{
    let mut diffs = Vec::new();
    if ctx.too_deep() {
        return Ok(diffs);
    }
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
                let entry_diffs = sync_diff_rec(
                    ctx.deeper(),
                    &src_root,
                    &dst_root,
                    file.join(entry.file_name()),
                )?;
                diffs.extend(entry_diffs.into_iter());
            }
        }
        (FileType::File, FileType::Dir) | (FileType::None, FileType::Dir) => {
            for entry in dst.read_dir()? {
                let entry: DirEntry = entry?;
                let entry_diffs = sync_diff_rec(
                    ctx.deeper(),
                    &src_root,
                    &dst_root,
                    file.join(entry.file_name()),
                )?;
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
                let entry_diffs = sync_diff_rec(ctx.deeper(), &src_root, &dst_root, file)?;
                diffs.extend(entry_diffs.into_iter());
            }
        }
        (FileType::None, FileType::None) => {}
    }

    Ok(diffs)
}

// Compute diff between two folders
// Returned path will be relative
pub fn sync_diff<A, B>(src: A, dst: B, settings: &SyncSettings) -> Result<Vec<Diff>>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let src: &Path = src.as_ref();
    let dst: &Path = dst.as_ref();
    if !src.exists() {
        return Err(io::Error::from(io::ErrorKind::NotFound).into());
    }
    let ctx = SyncContext {
        current_depth: 0,
        settings,
    };
    let mut diffs = sync_diff_rec(ctx, src, dst, "")?;
    for exclude in &settings.exclude {
        diffs.retain(|p| !exclude.matches_path(&p.path));
    }
    Ok(diffs)
}

// TODO: add option for progress
// TODO: support symlink

// One way sync from src to dst
pub fn sync<A, B>(src: A, dst: B, diffs: &[Diff]) -> Result<()>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let src = src.as_ref();
    let dst = dst.as_ref();
    if src.is_dir() {
        fs::create_dir_all(dst)?;
    } else if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    for diff in diffs {
        let mut src_path = src.to_owned();
        let mut dst_path = dst.to_owned();
        if diff.path().parent().is_some() {
            src_path = src_path.join(diff.path());
            dst_path = dst_path.join(diff.path());
        }
        match diff.kind() {
            DiffKind::Modified => {
                fs::copy(src_path, dst_path)?;
            }
            DiffKind::Added => {
                if src_path.is_dir() {
                    fs::create_dir_all(dst_path)?;
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
