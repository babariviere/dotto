mod add;
mod git;
mod init;

pub use self::add::*;
pub use self::git::*;
pub use self::init::*;

use crate::config::{Config, Context};
use crate::error::Result;

pub trait Command {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()>;
}
