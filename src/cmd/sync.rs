//! For testing purpose

use super::Command;
use crate::config::{Config, Context};
use crate::error::Result;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct SyncCmd {
    #[structopt(parse(from_os_str))]
    src: PathBuf,
    #[structopt(parse(from_os_str))]
    dst: PathBuf,
}

impl Command for SyncCmd {
    fn run(&self, _ctx: &Context, _config: &mut Config) -> Result<()> {
        let diffs = crate::sync::sync_diff(&self.src, &self.dst)?;
        println!("change list:");
        for diff in &diffs {
            println!("{}", diff);
        }
        write!(io::stdout(), "confirm changes [y/N]: ")?;
        io::stdout().flush()?;
        let mut c = [0];
        io::stdin().read(&mut c)?;
        if c[0] as char == 'y' {
            crate::sync::sync(&self.src, &self.dst, &diffs)?;
            let diffs = crate::sync::sync_diff(&self.src, &self.dst)?;
            println!("==> checking diff");
            if diffs.is_empty() {
                println!("==> everything is synced");
            } else {
                println!("==> error: here is the list of not synced item");
                for diff in diffs {
                    println!("{}", diff);
                }
            }
        } else {
            println!("==> cancelled");
        }
        Ok(())
    }
}
