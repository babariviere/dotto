use super::Command;
use crate::config::{Config, Context};
use crate::error::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct AddCmd {
    #[structopt(name = "files", parse(from_os_str))]
    files: Vec<PathBuf>,
    #[structopt(short = "r", long = "recursive")]
    recursive: bool,
}

impl Command for AddCmd {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()> {
        for file in &self.files {
            println!("==> adding {}", file.display());
            config.add_file(ctx, file, self.recursive)?;
        }
        Ok(())
    }
}
