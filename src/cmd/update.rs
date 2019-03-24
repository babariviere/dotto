use super::{Command, SyncContext};
use crate::config::{Config, Context};
use crate::error::Result;
use crate::sync::{self, SyncSettings};
use std::io::{self, Read, Write};
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UpdateCmd {}

impl Command for UpdateCmd {
    fn run(&self, ctx: &Context, config: &mut Config) -> Result<()> {
        let mut sync_ctx = Vec::new();
        for file in &config.files {
            let src_root = ctx.get_path(&file.location).join(&file.path);
            let dst_root = ctx.dot.join(&file.path);
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
                let src_root = ctx.get_path(&sctx.location).join(&sctx.path);
                let dst_root = ctx.dot.join(&sctx.path);
                println!("==> updating {}", dst_root.display());
                sync::sync(src_root, dst_root, &sctx.diffs)?;
            }
            if let Some(git) = config.git_dir(ctx).as_ref() {
                let mut message = String::new();
                for sctx in &sync_ctx {
                    for diff in &sctx.diffs {
                        // TODO: do not join when root is file
                        // maybe we should a small function to manage this case?
                        message.push_str(&format!(
                            "- {} {}\n",
                            diff.kind(),
                            sctx.path.join(diff.path()).display()
                        ));
                    }
                }
                process::Command::new("git")
                    .arg("--git-dir")
                    .arg(git)
                    .arg("--work-tree")
                    .arg(&ctx.dot)
                    .arg("add")
                    .arg("-A")
                    .status()?;
                let mut proc = process::Command::new("git")
                    .arg("--git-dir")
                    .arg(git)
                    .arg("--work-tree")
                    .arg(&ctx.dot)
                    .arg("commit")
                    .arg("-F")
                    .arg("-")
                    .stdin(process::Stdio::piped())
                    .spawn()?;
                proc.stdin.as_mut().unwrap().write_all(message.as_bytes())?;
                proc.wait()?;
            }
        } else {
            println!("==> cancelled");
        }
        Ok(())
    }
}
