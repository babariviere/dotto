use super::{Command, SyncContext};
use crate::config::{Config, Context};
use crate::error::Result;
use crate::sync::{self, SyncSettings};
use std::io::{self, Read, Write};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct InstallCmd {}

// TODO: allow to copy as root
impl Command for InstallCmd {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()> {
        let mut sync_ctx = Vec::new();
        for file in &config.files {
            let src_root = ctx.dot.join(&file.path);
            let dst_root = ctx.get_path(&file.location).join(&file.path);
            let file_diffs = sync::sync_diff(
                src_root,
                &dst_root,
                &SyncSettings::new(0, file.recursive, file.exclude.as_slice())?,
            )?;
            if file_diffs.is_empty() {
                continue;
            }
            sync_ctx.push(SyncContext::new(
                file.location.clone(),
                file.path.clone(),
                file_diffs,
            ));
        }
        if sync_ctx.is_empty() {
            println!("==> everything is up to date");
            return Ok(());
        }
        println!("==> these changes will be applied:");
        for sctx in &sync_ctx {
            println!(
                "  - in {}:",
                ctx.get_path(&sctx.location).join(&sctx.path).display()
            );
            for diff in &sctx.diffs {
                println!("    - {}", diff);
            }
        }
        let mut lock = io::stdout();
        write!(lock, "==> confirm? [y/N]: ")?;
        lock.flush()?;
        let mut c = [0];
        io::stdin().read(&mut c)?;
        if c[0] as char == 'y' {
            for sctx in &sync_ctx {
                let dst_root = ctx.get_path(&sctx.location).join(&sctx.path);
                let src_root = ctx.dot.join(&sctx.path);
                println!("==> installing into {}", dst_root.display());
                sync::sync(src_root, dst_root, &sctx.diffs)?;
            }
        } else {
            println!("==> cancelled");
        }
        Ok(())
    }
}
