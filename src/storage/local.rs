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

impl Storage for Local {
    fn open<E: AsRef<Entry>>(&mut self, entry: E) -> Result<Box<dyn VFile>> {
        let _entry = entry.as_ref();
        unimplemented!()
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
        println!("{:?}", entry);
        println!("{:?}", self.path(entry));
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
    use super::Local;
    use crate::storage::tests::*;

    const PATH_PREFIX: &str = "/tmp/dotto_test_storage_local";

    /// Test storage creation capability
    #[test]
    pub fn test_create() {
        std::fs::create_dir_all(PATH_PREFIX);
        let mut local = Local::new(PATH_PREFIX);
        create(&mut local);
        std::fs::remove_dir_all(PATH_PREFIX);
    }

    /// Test storage opening capacity

    pub fn test_open() {}
}
