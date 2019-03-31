mod local;

pub use self::local::*;

use crate::error::Result;
use std::path::{Component, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct VPathPart {
    key: String,
}

impl VPathPart {
    pub fn new<S: Into<String>>(key: S) -> VPathPart {
        VPathPart { key: key.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VPath {
    parts: Vec<VPathPart>,
    absolute: bool,
}

impl VPath {
    pub fn new<S: AsRef<str>>(path: S) -> VPath {
        let path = path.as_ref();
        let absolute = path.chars().next().map(|c| c == '/').unwrap_or(false);
        let parts = path.split('/').map(VPathPart::new).collect();
        VPath { parts, absolute }
    }

    pub fn parent(&mut self) -> &mut VPath {
        self.parts.pop();
        self
    }

    pub fn path(&self) -> PathBuf {
        let mut buf = String::new();
        if self.absolute {
            buf.push('/');
        }
        PathBuf::from(
            self.parts
                .iter()
                .fold(buf, |a, b| format!("{}/{}", a, b.key)),
        )
    }
}

impl From<PathBuf> for VPath {
    fn from(p: PathBuf) -> VPath {
        let mut path = VPath {
            parts: Vec::new(),
            absolute: false,
        };
        for component in p.components() {
            match component {
                Component::RootDir => path.absolute = true,
                Component::Normal(s) => {
                    if let Some(s) = s.to_str() {
                        path.parts.push(VPathPart::new(s));
                    }
                }
                Component::CurDir => {} // ignore it
                Component::ParentDir => {
                    path.parent();
                }
                Component::Prefix(_) => {}
            }
        }
        path
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntryKind {
    Dir,
    File,
    Link(VPath),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub kind: EntryKind,
    pub vpath: VPath,
}

impl Entry {
    pub fn new(kind: EntryKind, vpath: VPath) -> Entry {
        Entry { kind, vpath }
    }
}

pub trait Storage {
    // list all files in virtual directory
    fn list(&self, vpath: &VPath) -> Result<Vec<Entry>>;

    // compare both entries with a checksum
    fn checksum(&self, a: &Entry, b: &Entry) -> Result<bool>;

    // get entry kind for path
    fn entry_kind(&self, vpath: &VPath, check_link: bool) -> Result<EntryKind>;

    // create an entry
    fn create(&mut self, entry: &Entry) -> Result<()>;

    // copy src entry to dst entry
    fn copy(&mut self, src: &Entry, dst: &Entry) -> Result<()>;

    // remove an entry
    fn remove(&mut self, entry: &Entry) -> Result<()>;
}
