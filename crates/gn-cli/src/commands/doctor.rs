use anyhow::Result;
use clap::Args;
use std::path::Path;
use std::process::Command;

/// Run diagnostic checks on the git-notes environment
#[derive(Args)]
pub struct DoctorArgs {}

pub fn run(_args: &DoctorArgs) -> Result<()> {
    println!("🩺 git-notes doctor - running diagnostic checks...\n");

    let mut all_passed = true;

    // 1. Check remote notes ref config
    print!("Checking remote notes ref config... ");
    let fetch_notes = Command::new("git")
        .args(["config", "--get", "remote.origin.fetch"])
        .output()?;
    let fetch_notes_out = String::from_utf8_lossy(&fetch_notes.stdout);
    if fetch_notes_out.contains("refs/notes/*:refs/notes/*") {
        println!("✅ Configured");
    } else {
        println!("❌ Missing");
        println!(
            "  💡 Fix: Run `git config --add remote.origin.fetch '+refs/notes/*:refs/notes/*'`"
        );
        all_passed = false;
    }

    // 2. Check hook status
    print!("Checking hook status... ");
    let git_dir_cmd = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?;
    let git_dir = String::from_utf8_lossy(&git_dir_cmd.stdout)
        .trim()
        .to_string();
    let hook_path = Path::new(&git_dir).join("hooks").join("post-commit");

    // We just check if there is ANY post-commit hook for now,
    // ideally we'd check if it's our hook.
    if hook_path.exists() {
        println!("✅ Installed");
    } else {
        println!("❌ Missing");
        println!(
            "  💡 Fix: Run `git-notes-hooks install` or add to your post-commit hook manually."
        );
        all_passed = false;
    }

    // 3. Stale note detection
    print!("Checking stale sync... ");
    let status_cmd = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    let status = String::from_utf8_lossy(&status_cmd.stdout);
    if status.trim().is_empty() {
        println!("✅ Up to date");
    } else {
        println!("⚠️ Uncommitted changes (sync might be stale)");
        all_passed = false;
    }

    println!();
    if all_passed {
        println!("🎉 All checks passed! Your git-notes environment is healthy.");
    } else {
        println!("🚨 Some checks failed. See the tips above to fix them.");
    }

    Ok(())
}
