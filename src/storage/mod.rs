pub mod local;

use std::io::{Read, Write};
use std::path;

/// A virtual path in storage.
#[derive(Clone, Debug, Eq, PartialOrd, PartialEq, Ord)]
pub struct VPath {
    root: bool,
    path: String,
}

impl VPath {
    /// create a system path from VPath
    fn system_path(&self) -> path::PathBuf {
        path::Path::new(&self.path).to_owned()
    }

    /// join path with another VPath
    fn join<P: AsRef<VPath>>(&self, rhs: P) -> VPath {
        let rhs = rhs.as_ref();

        let mut new = self.clone();
        if !new.path.ends_with('/') && !rhs.path.starts_with('/') {
            new.path.push('/');
        }
        new.path.extend(rhs.path.chars());
        new
    }
}

impl From<&str> for VPath {
    fn from(v: &str) -> Self {
        VPath {
            root: v.starts_with('/'),
            path: v.to_owned(),
        }
    }
}

impl From<path::PathBuf> for VPath {
    fn from(p: path::PathBuf) -> Self {
        VPath {
            root: p.is_absolute(),
            path: p.display().to_string(),
        }
    }
}

impl From<&path::Path> for VPath {
    fn from(p: &path::Path) -> Self {
        VPath {
            root: p.is_absolute(),
            path: p.display().to_string(),
        }
    }
}

impl Into<path::PathBuf> for VPath {
    fn into(self) -> path::PathBuf {
        self.system_path()
    }
}

impl AsRef<path::Path> for VPath {
    fn as_ref(&self) -> &path::Path {
        self.path.as_ref()
    }
}

/// List of all different entries types.
#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq, Ord)]
pub enum EntryKind {
    /// A file
    File,
    /// A directory that can contains multiple entry
    Dir,
}

/// File entry in storage
#[derive(Clone, Debug, Eq, PartialOrd, PartialEq, Ord)]
pub struct Entry {
    path: VPath,
    kind: EntryKind,
}

impl AsRef<Entry> for Entry {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Entry {
    /// Create a new entry
    pub fn new<P: Into<VPath>>(path: P, kind: EntryKind) -> Entry {
        Entry {
            path: path.into(),
            kind: kind,
        }
    }

    /// Create a new entry of type directory
    pub fn new_dir<P: Into<VPath>>(path: P) -> Entry {
        Entry::new(path, EntryKind::Dir)
    }

    /// Create a new entry of type file
    pub fn new_file<P: Into<VPath>>(path: P) -> Entry {
        Entry::new(path, EntryKind::File)
    }

    /// Returns virtual path
    pub fn path(&self) -> &VPath {
        &self.path
    }

    /// Returns entry kind
    pub fn kind(&self) -> EntryKind {
        self.kind
    }
}

impl AsRef<VPath> for Entry {
    fn as_ref(&self) -> &VPath {
        &self.path
    }
}

/// Virtual file handler returned while opening file in storage.
pub trait VFile: Read + Write {}

/// List of all errors that can be returned by Storage.
#[derive(Debug)]
pub enum Error {
    /// File is not found
    NotFound(VPath),
    /// Not a file
    NotAFile(VPath),
    /// Invalid copy
    InvalidCopy(EntryKind, EntryKind),

    /// std::io error
    Io(std::io::Error, VPath),
}

impl Error {
    /// Convert std::io Error into custom Error
    pub fn from_io(error: std::io::Error, path: VPath) -> Error {
        use std::io::ErrorKind;
        match error.kind() {
            ErrorKind::NotFound => Error::NotFound(path),
            _ => Error::Io(error, path),
        }
    }
}

/// Result returned by Storage functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Storage is an abstraction to access files and directories.
/// It implements all basic method to interact with a file system.
pub trait Storage {
    /// Open a file in the storage. If entry is a directory, this will returns an error.
    // TODO: add permissions
    fn open<E: AsRef<Entry>>(&mut self, entry: E) -> Result<Box<dyn VFile>>;

    /// Create a file/directory in the storage. This will returns nothing, only an error if any.
    fn create<E: AsRef<Entry>>(&mut self, entry: E) -> Result<()>;

    /// Remove a file/directory in the storage. This is not recursive.
    fn remove<E: AsRef<Entry>>(&mut self, entry: E) -> Result<()>;

    /// Copy entry to a new entry.
    fn copy<D: AsRef<Entry>, S: AsRef<Entry>>(&mut self, src: S, dst: D) -> Result<()>;

    /// List all files in directory. Returns nothing if it's a file.
    fn list<E: AsRef<Entry>>(&self, entry: E) -> Result<Vec<Entry>>;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::io::{Read, Write};

    /// Test storage creation capability
    pub fn create<S: Storage>(storage: &mut S) {
        let entry_a = Entry::new_file("a");
        let entry_b = Entry::new_dir("b");
        let entry_c = Entry::new_dir("c");
        assert!(storage.create(&entry_a).is_ok());
        assert!(storage.create(&entry_b).is_ok());
        assert!(storage.create(&entry_c).is_ok());

        let root = Entry::new_dir("/");
        let res = storage.list(&root);
        assert!(res.is_ok());
        let mut res = res.unwrap();
        res.sort();
        assert_eq!(res, vec![entry_a, entry_b, entry_c]);
    }

    /// Test storage opening capacity
    pub fn open<S: Storage>(storage: &mut S) {
        let entry_d = Entry::new_file("d");
        assert!(storage.create(&entry_d).is_ok());
        let content = "hello world";
        let mut file_ref = storage.open(&entry_d).unwrap();
        assert!(file_ref.write(content.as_bytes()).is_ok());
        let mut file_ref = storage.open(&entry_d).unwrap();
        let mut buf = String::new();
        assert!(file_ref.read_to_string(&mut buf).is_ok());
        assert_eq!(content, buf);
    }
}