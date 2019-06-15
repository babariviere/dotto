//! Local storage implementation
use super::*;

use std::fs;

// TODO: use root

/// Local filesystem implementation.
/// Use root as a prefix for all virtual path.
pub struct Local {
    root: VPath,
}

impl Local {
    /// Creates a new local storage
    pub fn new<P: Into<VPath>>(root: P) -> Local {
        Local { root: root.into() }
    }

    // prepend root path to current path
    fn path<P: AsRef<VPath>>(&self, p: P) -> VPath {
        self.root.join(p)
    }
}

impl VFile for std::fs::File {}

impl Storage for Local {
    fn open<E: AsRef<Entry>>(&mut self, entry: E) -> Result<Box<dyn VFile>> {
        let entry = entry.as_ref();
        if entry.kind() != EntryKind::File {
            return Err(Error::NotAFile(entry.path().clone()));
        }
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.path(entry))
            .map_err(|err| Error::from_io(err, entry.path().clone()))?;
        Ok(Box::new(f))
    }

    fn create<E: AsRef<Entry>>(&mut self, entry: E) -> Result<()> {
        let entry = entry.as_ref();
        match entry.kind() {
            EntryKind::File => {
                fs::File::create(self.path(entry))
                    .map_err(|err| Error::from_io(err, entry.path().clone()))?;
            }
            EntryKind::Dir => {
                fs::create_dir(self.path(entry))
                    .map_err(|err| Error::from_io(err, entry.path().clone()))?;
            }
        }
        Ok(())
    }

    fn remove<E: AsRef<Entry>>(&mut self, entry: E) -> Result<()> {
        let entry = entry.as_ref();
        match entry.kind() {
            EntryKind::File => {
                fs::remove_file(self.path(entry))
                    .map_err(|err| Error::from_io(err, entry.path().clone()))?;
            }
            EntryKind::Dir => {
                fs::remove_dir(self.path(entry))
                    .map_err(|err| Error::from_io(err, entry.path().clone()))?;
            }
        }
        Ok(())
    }

    fn copy<D: AsRef<Entry>, S: AsRef<Entry>>(&mut self, src: S, dst: D) -> Result<()> {
        let dst = dst.as_ref();
        let src = src.as_ref();
        match (dst.kind(), src.kind()) {
            (EntryKind::File, EntryKind::File) => {
                fs::copy(self.path(src), self.path(dst))
                    .map_err(|err| Error::from_io(err, dst.path().clone()))?;
            }
            (EntryKind::Dir, EntryKind::Dir) => {
                fs::create_dir(self.path(dst))
                    .map_err(|err| Error::from_io(err, dst.path().clone()))?;
            }
            (from, to) => {
                return Err(Error::InvalidCopy(from, to));
            }
        }
        Ok(())
    }

    fn list<E: AsRef<Entry>>(&self, entry: E) -> Result<Vec<Entry>> {
        let entry = entry.as_ref();
        match entry.kind() {
            EntryKind::File => {
                return Ok(vec![entry.clone()]);
            }
            EntryKind::Dir => fs::read_dir(self.path(entry))
                .map_err(|err| Error::from_io(err, entry.path().clone()))?
                .filter_map(std::result::Result::ok)
                .map(|entry| {
                    let meta = entry
                        .metadata()
                        .map_err(|err| Error::from_io(err, entry.path().into()))?;
                    let kind = if meta.is_dir() {
                        EntryKind::Dir
                    } else {
                        EntryKind::File
                    };
                    let path = entry.path();
                    let path = path
                        .strip_prefix(&self.root)
                        .expect("prefix should be correct");
                    Ok(Entry::new(path, kind))
                })
                .collect(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    #![allow(unused_must_use)]
    use super::Local;
    use crate::storage::tests::*;

    const PATH_PREFIX: &str = "/tmp/dotto_test_storage_local";

    fn run_test<F: Fn(&mut Local)>(name: &str, f: F) {
        let path = format!("{}/{}", PATH_PREFIX, name);
        // asserts files are deleted
        std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path);
        let mut local = Local::new(&path);
        f(&mut local);
        std::fs::remove_dir_all(&path);
    }

    /// Test storage creation capability
    #[test]
    pub fn test_create() {
        run_test("create", create);
    }

    /// Test storage opening capacity
    #[test]
    pub fn test_open() {
        run_test("open", open);
    }

    /// Test storage removing capacity
    #[test]
    pub fn test_remove() {
        run_test("remove", remove);
    }

    /// Test storage copying capacity
    #[test]
    pub fn test_copy() {
        run_test("copy", copy);
    }

    /// Test storage listing capacity
    #[test]
    pub fn test_list() {
        run_test("list", list);
    }

}
