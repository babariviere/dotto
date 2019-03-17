mod cmd;
mod config;
mod error;
mod sync;

use crate::cmd::*;
use std::path::PathBuf;
use structopt::StructOpt;

// TODO: install and update

#[derive(Debug, StructOpt)]
enum CliCommand {
    /// Add file(s) to dot index
    #[structopt(name = "add")]
    Add(AddCmd),
    /// Initialize dot configuration and folder
    #[structopt(name = "init")]
    Init(InitCmd),
    /// Open config in your editor ($EDITOR by default)
    #[structopt(name = "edit")]
    Edit(EditCmd),
    /// Launch git command from dot directory
    #[structopt(name = "git")]
    Git(GitCmd),
    /// Synchronize folders (only for testing)
    #[structopt(name = "sync")]
    Sync(SyncCmd),
}

/// Dotfiles manager
#[derive(Debug, StructOpt)]
#[structopt(name = "dot")]
struct Cli {
    /// Path to dot config file, use $DOT_PATH/config.yml by default
    #[structopt(name = "config", short = "c", long = "config", parse(from_os_str))]
    config: Option<PathBuf>,

    #[structopt(subcommand)]
    command: CliCommand,
}

fn main() -> error::Result<()> {
    let args = Cli::from_args();
    let mut context = config::Context::new();
    if let Some(path) = &args.config {
        context.dot_config = path.to_owned();
    }
    let mut config = match config::Config::open(&context.dot_config) {
        Ok(c) => c,
        Err(e) => {
            if let CliCommand::Init(_) = &args.command {
                config::Config::default()
            } else {
                return Err(e);
            }
        }
    };
    match args.command {
        CliCommand::Add(a) => a.run(&context, &mut config)?,
        CliCommand::Init(i) => i.run(&context, &mut config)?,
        CliCommand::Edit(e) => e.run(&context, &mut config)?,
        CliCommand::Git(g) => g.run(&context, &mut config)?,
        CliCommand::Sync(s) => s.run(&context, &mut config)?,
    }
    config.save(&context.dot_config)?;
    Ok(())
}
