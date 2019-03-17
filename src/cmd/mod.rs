mod add;
mod edit;
mod git;
mod init;
mod install;
mod sync;
mod update;

pub use self::add::*;
pub use self::edit::*;
pub use self::git::*;
pub use self::init::*;
pub use self::install::*;
pub use self::sync::*;
pub use self::update::*;

use crate::config::{Config, Context, Location};
use crate::error::Result;
use crate::sync::Diff;
use std::path::PathBuf;

pub trait Command {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()>;
}

pub struct SyncContext {
    pub location: Location,
    pub path: PathBuf,
    pub diffs: Vec<Diff>,
}

impl SyncContext {
    pub fn new(location: Location, path: PathBuf, diffs: Vec<Diff>) -> SyncContext {
        SyncContext {
            location,
            path,
            diffs,
        }
    }
}
