use super::Command;
use crate::config::{Config, Context};
use crate::error::Result;
use std::env;
use std::process;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct EditCmd {
    /// Binary name/path to use as an editor
    #[structopt(name = "editor")]
    editor: Option<String>,
}

impl Command for EditCmd {
    fn run(&self, ctx: &Context, _config: &mut Config) -> Result<()> {
        let editor = self
            .editor
            .clone()
            .ok_or(|| env::VarError::NotPresent)
            .or_else(|_| env::var("EDITOR"))
            .map_err(failure::Error::from)?;
        process::Command::new(editor)
            .arg(&ctx.dot_config)
            .status()?;
        Ok(())
    }
}
