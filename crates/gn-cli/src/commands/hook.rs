use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Args, Debug)]
pub struct HookArgs {
    #[command(subcommand)]
    pub action: HookAction,
}

#[derive(Subcommand, Debug)]
pub enum HookAction {
    /// Install auto-sync hooks into local .git/hooks/
    Install,
    /// Remove auto-sync hooks from local .git/hooks/
    Uninstall,
}

pub fn run(args: &HookArgs) -> Result<()> {
    let root_out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to check git repository")?;

    if !root_out.status.success() {
        anyhow::bail!("Not inside a Git repository.");
    }

    let repo_root = String::from_utf8_lossy(&root_out.stdout).trim().to_string();
    let hooks_dir = Path::new(&repo_root).join(".git").join("hooks");

    let post_merge = hooks_dir.join("post-merge");
    let pre_push = hooks_dir.join("pre-push");

    match args.action {
        HookAction::Install => {
            fs::create_dir_all(&hooks_dir)?;
            let pm_script = "#!/bin/sh\n# git-notes auto-sync hook\ngit-notes sync pull --quiet 2>/dev/null || true\n";
            let pp_script = "#!/bin/sh\n# git-notes auto-sync hook\ngit-notes sync push --quiet 2>/dev/null || true\n";

            fs::write(&post_merge, pm_script)?;
            fs::write(&pre_push, pp_script)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&post_merge)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&post_merge, perms)?;

                let mut perms2 = fs::metadata(&pre_push)?.permissions();
                perms2.set_mode(0o755);
                fs::set_permissions(&pre_push, perms2)?;
            }

            println!("\x1b[32m✔\x1b[0m Successfully installed auto-sync hooks in .git/hooks/");
        }
        HookAction::Uninstall => {
            if post_merge.exists() {
                let content = fs::read_to_string(&post_merge).unwrap_or_default();
                if content.contains("git-notes") {
                    fs::remove_file(&post_merge)?;
                }
            }
            if pre_push.exists() {
                let content = fs::read_to_string(&pre_push).unwrap_or_default();
                if content.contains("git-notes") {
                    fs::remove_file(&pre_push)?;
                }
            }
            println!("\x1b[32m✔\x1b[0m Removed git-notes hooks from .git/hooks/");
        }
    }

    Ok(())
}
