use super::Command;
use crate::config::{Config, Context};
use crate::error::Result;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

// Initialize dot directory
#[derive(Debug, StructOpt)]
pub struct InitCmd {
    // Initalize an empty git repo
    #[structopt(long = "git")]
    git: bool,
    // Specify git working directory to make all git request into
    // Override --git if enabled
    #[structopt(long = "git-dir", parse(from_os_str))]
    git_dir: Option<PathBuf>,
}

impl Command for InitCmd {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()> {
        println!("==> creating {}", ctx.dot.display());
        std::fs::create_dir_all(&ctx.dot)?;
        if let Some(gd) = &self.git_dir {
            config.set_git_dir(gd);
        } else if self.git {
            println!("==> initializing git directory");
            process::Command::new("git")
                .arg("-C")
                .arg(ctx.dot.display().to_string())
                .arg("init")
                .status()?;
        }
        Ok(())
    }
}
