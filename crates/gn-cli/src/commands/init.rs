use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("\n\x1b[1;36m🚀 Initializing git-notes in repository...\x1b[0m\n");

    // 1. Verify git repo
    let root_out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to check git repository")?;

    if !root_out.status.success() {
        anyhow::bail!("Not inside a Git repository. Run 'git init' first.");
    }

    let repo_root = String::from_utf8_lossy(&root_out.stdout).trim().to_string();

    // 2. Configure remote origin fetch refspec for refs/notes/*
    let remote_check = Command::new("git").args(["remote"]).output()?;
    let remotes = String::from_utf8_lossy(&remote_check.stdout);

    if remotes.lines().any(|r| r.trim() == "origin") {
        let fetch_check = Command::new("git")
            .args(["config", "--get-all", "remote.origin.fetch"])
            .output()?;
        let current_fetches = String::from_utf8_lossy(&fetch_check.stdout);

        if !current_fetches.contains("refs/notes/*") {
            Command::new("git")
                .args(["config", "--add", "remote.origin.fetch", "+refs/notes/*:refs/notes/*"])
                .status()?;
            println!("  \x1b[32m✔\x1b[0m Configured remote.origin.fetch for refs/notes/*");
        } else {
            println!("  \x1b[32m✔\x1b[0m remote.origin.fetch already configured");
        }
    } else {
        println!("  \x1b[33mℹ\x1b[0m No remote 'origin' found. Skipping refspec config for now.");
    }

    // 3. Install native git hooks (post-merge & pre-push)
    let hooks_dir = Path::new(&repo_root).join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    let post_merge_path = hooks_dir.join("post-merge");
    let pre_push_path = hooks_dir.join("pre-push");

    let post_merge_script = "#!/bin/sh\n# git-notes auto-sync hook\ngit-notes sync pull --quiet 2>/dev/null || true\n";
    let pre_push_script = "#!/bin/sh\n# git-notes auto-sync hook\ngit-notes sync push --quiet 2>/dev/null || true\n";

    fs::write(&post_merge_path, post_merge_script)?;
    fs::write(&pre_push_path, pre_push_script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&post_merge_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&post_merge_path, perms)?;

        let mut perms2 = fs::metadata(&pre_push_path)?.permissions();
        perms2.set_mode(0o755);
        fs::set_permissions(&pre_push_path, perms2)?;
    }

    println!("  \x1b[32m✔\x1b[0m Installed git auto-sync hooks (.git/hooks/post-merge & pre-push)");

    println!("\n\x1b[32m✨ git-notes initialized successfully!\x1b[0m");
    println!("Try running: \x1b[36mgn l\x1b[0m or \x1b[36mgn a -f <file> -l <line> -m \"...\"\x1b[0m\n");

    Ok(())
}
