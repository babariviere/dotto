use super::Command;
use crate::config::{Config, Context};
use crate::error::Result;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GitCmd {}

impl Command for GitCmd {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()> {
        let git_dir = match config.git_dir(ctx) {
            Some(g) => g,
            None => return Err(crate::error::DotError::NoGitDir.into()),
        };
        let m = crate::Cli::clap().get_matches();
        let git = m.subcommand_matches("git").unwrap();
        let mut svalues = Vec::new();
        if let Some(v) = git.values_of("") {
            svalues = v.collect::<Vec<&str>>();
        }
        match git.subcommand() {
            (cmd, Some(sub_m)) => {
                let mut evalues = Vec::new();
                if let Some(v) = sub_m.values_of("") {
                    evalues = v.collect::<Vec<&str>>();
                }
                process::Command::new("git")
                    .arg("--git-dir")
                    .arg(git_dir)
                    .arg("--work-tree")
                    .arg(&ctx.dot)
                    .args(svalues)
                    .arg(cmd)
                    .args(evalues)
                    .status()?;
            }
            _ => {}
        }
        Ok(())
    }
}
