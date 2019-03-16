use super::Command;
use crate::config::{Config, Context};
use crate::error::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GitCmd {
    #[structopt(name = "args")]
    args: Vec<String>,
}

impl Command for GitCmd {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()> {
        unimplemented!()
    }
}
