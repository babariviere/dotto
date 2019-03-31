use super::{Entry, EntryKind, Storage, VPath};
use crate::error::{DotError, Result};
use std::fs;
use std::io::Read;

pub struct LocalStorage;

impl Storage for LocalStorage {
    fn list(&self, vpath: &VPath) -> Result<Vec<Entry>> {
        let path = vpath.path();
        let mut entries = Vec::new();
        for entry in path.read_dir()? {
            let entry = entry?;
            let vpath = VPath::from(entry.path());
            let kind = self.entry_kind(&vpath, true)?;
            entries.push(Entry::new(kind, vpath));
        }
        Ok(entries)
    }

    fn checksum(&self, a: &Entry, b: &Entry) -> Result<bool> {
        if a.kind == EntryKind::Dir || b.kind == EntryKind::Dir {
            return Err(DotError::ChecksumDir.into());
        }
        let mut fa = fs::File::open(a.vpath.path())?;
        let mut fb = fs::File::open(b.vpath.path())?;
        let mut ba = Vec::new();
        let mut bb = Vec::new();
        fa.read_to_end(&mut ba)?;
        fb.read_to_end(&mut bb)?;

        let suma = md5::compute(ba);
        let sumb = md5::compute(bb);
        Ok(suma == sumb)
    }

    fn entry_kind(&self, vpath: &VPath, check_link: bool) -> Result<EntryKind> {
        let path = vpath.path();
        if !path.exists() {
            return Err(DotError::NotFound(path.display().to_string()).into());
        }
        if check_link {
            if let Ok(p) = path.read_link() {
                return Ok(EntryKind::Link(VPath::from(p)));
            }
        }
        if path.is_dir() {
            return Ok(EntryKind::Dir);
        }
        return Ok(EntryKind::File);
    }

    fn create(&mut self, entry: &Entry) -> Result<()> {
        let path = entry.vpath.path();
        match &entry.kind {
            EntryKind::Dir => fs::create_dir(&path)?,
            EntryKind::File => {
                fs::File::create(&path)?;
            }
            EntryKind::Link(dst) => {
                let dst = dst.path();
                std::os::unix::fs::symlink(&path, &dst)?
            }
        }
        Ok(())
    }

    fn copy(&mut self, src: &Entry, dst: &Entry) -> Result<()> {
        if src.kind != dst.kind {
            return Err(DotError::InvalidCopy.into());
        }
        let src_path = src.vpath.path();
        let dst_path = dst.vpath.path();
        match src.kind {
            EntryKind::Dir => fs::create_dir(&dst_path)?,
            EntryKind::File => {
                fs::copy(&src_path, &dst_path)?;
            }
            EntryKind::Link(_) => {} // TODO: what should we do ?
        }
        Ok(())
    }

    fn remove(&mut self, entry: &Entry) -> Result<()> {
        let path = entry.vpath.path();
        match entry.kind {
            EntryKind::Dir => fs::remove_dir(&path)?,
            EntryKind::File => fs::remove_file(&path)?,
            EntryKind::Link(_) => fs::remove_file(&path)?,
        }
        Ok(())
    }
}
