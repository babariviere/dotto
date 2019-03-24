use super::Command;
use crate::config::{Config, Context};
use crate::error::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ExcludeCmd {
    #[structopt(name = "excludes")]
    excludes: Vec<String>,
}

impl Command for ExcludeCmd {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()> {
        for exclude in &self.excludes {
            println!("==> adding exclusion {}", exclude);
            if let Err(e) = config.add_exclude(ctx, &exclude) {
                println!("!=> {}", e);
            }
        }
        Ok(())
    }
}
