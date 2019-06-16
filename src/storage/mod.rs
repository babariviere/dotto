pub mod local;

use failure::Fail;
use std::fmt;
use std::io::{Read, Write};
use std::path;

/// A virtual path in storage.
#[derive(Clone, Debug, Eq, PartialOrd, PartialEq, Ord)]
pub struct VPath {
    root: bool,
    path: String,
}

impl VPath {
    /// create a new VPath from str
    pub fn new<S: AsRef<str>>(path: S) -> VPath {
        let path = path.as_ref();
        let root = path.starts_with('/');
        let mut parts = Vec::new();
        for p in path.split('/') {
            match p {
                "." => {}
                ".." => {
                    parts.pop();
                }
                p => {
                    parts.push(p);
                }
            }
        }
        let mut path = parts.join("/");
        if root {
            path = format!("/{}", path);
        }
        VPath { root, path }
    }

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

impl fmt::Display for VPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.path.fmt(f)
    }
}

impl From<&str> for VPath {
    fn from(v: &str) -> Self {
        VPath::new(v)
    }
}

impl From<&String> for VPath {
    fn from(v: &String) -> Self {
        VPath::new(v)
    }
}

impl From<path::PathBuf> for VPath {
    fn from(p: path::PathBuf) -> Self {
        VPath::new(p.display().to_string())
    }
}

impl From<&path::Path> for VPath {
    fn from(p: &path::Path) -> Self {
        VPath::new(p.display().to_string())
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

impl fmt::Display for EntryKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryKind::File => write!(f, "file"),
            EntryKind::Dir => write!(f, "dir"),
        }
    }
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
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "file not found {}", _0)]
    /// File is not found
    NotFound(VPath),
    #[fail(display = "{} is not a file", _0)]
    /// Not a file
    NotAFile(VPath),
    #[fail(display = "invalid copy from type {} to type {}", _0, _1)]
    /// Invalid copy
    InvalidCopy(EntryKind, EntryKind),

    #[fail(display = "{}: {}", _1, _0)]
    /// std::io error
    Io(#[fail(cause)] std::io::Error, VPath),
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
    #![allow(unused_must_use)]
    use super::*;

    use std::io::{Read, Write};

    #[test]
    fn test_vpath() {
        let path = "../../abc/def/ghi/../../.";
        let vpath = VPath::new(path);
        assert_eq!(vpath.path, "abc");
    }

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

    /// Test storage removing capacity
    pub fn remove<S: Storage>(storage: &mut S) {
        let entry_a = Entry::new_file("a");
        let entry_b = Entry::new_dir("b");
        let entry_c = Entry::new_dir("b/c");
        storage.create(&entry_a);
        storage.create(&entry_b);
        storage.create(&entry_c);

        assert!(storage.remove(entry_a).is_ok());
        assert!(storage.remove(&entry_b).is_err());
        assert!(storage.remove(entry_c).is_ok());
        assert!(storage.remove(entry_b).is_ok());
    }

    /// Test storage copying capacity
    pub fn copy<S: Storage>(storage: &mut S) {
        let entry_a = Entry::new_file("a");
        let entry_b = Entry::new_file("b");
        storage.create(&entry_a);

        let buf = "hello world";
        let mut f = storage.open(&entry_a).unwrap();
        f.write(buf.as_bytes()).unwrap();
        assert!(storage.copy(&entry_a, &entry_b).is_ok());
        let mut f = storage.open(&entry_b).unwrap();
        let mut readed = String::new();
        f.read_to_string(&mut readed).unwrap();
        assert_eq!(buf, readed);
    }

    /// Test storage listing capacity
    pub fn list<S: Storage>(storage: &mut S) {
        let entry_a = Entry::new_file("a");
        let entry_b = Entry::new_dir("b");
        let entry_c = Entry::new_dir("c");
        storage.create(&entry_a);
        storage.create(&entry_b);
        storage.create(&entry_c);

        let root = Entry::new_dir("/");
        let res = storage.list(&root);
        assert!(res.is_ok());
        let mut res = res.unwrap();
        res.sort();
        assert_eq!(res, vec![entry_a, entry_b, entry_c]);
    }

}
