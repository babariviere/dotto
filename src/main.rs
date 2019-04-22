mod cmd;
mod config;
mod error;
mod storage;
mod sync;

use crate::cmd::*;
use std::path::PathBuf;
use structopt::StructOpt;

// TODO: clean command to remove deleted projects / delete command
// TODO: add status command to get all diff and all

#[derive(Debug, StructOpt)]
enum CliCommand {
    /// Add file(s) to dot index
    #[structopt(name = "add")]
    Add(AddCmd),
    /// Open config in your editor ($EDITOR by default)
    #[structopt(name = "edit")]
    Edit(EditCmd),
    /// Exclude files (glob style: e.g src/**/*.rs) from install and update
    #[structopt(name = "exclude")]
    Exclude(ExcludeCmd),
    /// Run git in dot context
    #[structopt(
        name = "git",
        raw(setting = "structopt::clap::AppSettings::AllowExternalSubcommands")
    )]
    Git(GitCmd),
    /// Initialize dot configuration and folder
    #[structopt(name = "init")]
    Init(InitCmd),
    /// Install config files (warning: can delete files on system)
    #[structopt(name = "install")]
    Install(InstallCmd),
    /// Update dot directory with new changes
    #[structopt(name = "update")]
    Update(UpdateCmd),
}

/// Dotfiles manager
#[derive(Debug, StructOpt)]
#[structopt(name = "dotto")]
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
        CliCommand::Edit(e) => e.run(&context, &mut config)?,
        CliCommand::Exclude(e) => e.run(&context, &mut config)?,
        CliCommand::Git(g) => g.run(&context, &mut config)?,
        CliCommand::Init(i) => i.run(&context, &mut config)?,
        CliCommand::Install(i) => i.run(&context, &mut config)?,
        CliCommand::Update(u) => u.run(&context, &mut config)?,
    }
    config.save(&context.dot_config)?;
    Ok(())
}
