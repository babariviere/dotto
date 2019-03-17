mod cmd;
mod config;
mod error;
mod sync;

use crate::cmd::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum CliCommand {
    #[structopt(name = "add")]
    Add(AddCmd),
    #[structopt(name = "init")]
    Init(InitCmd),
    #[structopt(name = "git")]
    Git(GitCmd),
    #[structopt(name = "sync")]
    Sync(SyncCmd),
}

#[derive(Debug, StructOpt)]
struct Cli {
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
        CliCommand::Git(g) => g.run(&context, &mut config)?,
        CliCommand::Sync(s) => s.run(&context, &mut config)?,
    }
    config.save(&context.dot_config)?;
    Ok(())
}
