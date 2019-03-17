mod add;
mod edit;
mod git;
mod init;
mod sync;

pub use self::add::*;
pub use self::edit::*;
pub use self::git::*;
pub use self::init::*;
pub use self::sync::*;

use crate::config::{Config, Context};
use crate::error::Result;

pub trait Command {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()>;
}
