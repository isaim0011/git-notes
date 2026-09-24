use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("\n\x1b[1;36m🩺 git-notes doctor\x1b[0m");
    println!("Checking repository configuration and health...\n");

    let mut errors = 0;
    let mut warnings = 0;

    // 1. Git Repository Check
    let git_root = match Command::new("git").args(["rev-parse", "--show-toplevel"]).output() {
        Ok(out) if out.status.success() => {
            let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("  \x1b[32m✓\x1b[0m Git repository: {}", root);
            Some(root)
        }
        _ => {
            println!("  \x1b[31m✗\x1b[0m Git repository: not inside a valid git repo");
            errors += 1;
            None
        }
    };

    // 2. Binary Version Check
    let current_version = env!("CARGO_PKG_VERSION");
    println!("  \x1b[32m✓\x1b[0m Binary version: v{}", current_version);

    // 3. Notes Refs Namespaces Check
    let namespaces = ["comments", "review", "todos"];
    for ns in &namespaces {
        let ref_path = format!("refs/notes/{}", ns);
        let check_ref = Command::new("git")
            .args(["for-each-ref", &ref_path, "--format=%(objectname)"])
            .output();

        match check_ref {
            Ok(out) if out.status.success() && !out.stdout.is_empty() => {
                let note_count = Command::new("git")
                    .args(["notes", "--ref", &ref_path, "list"])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
                    .unwrap_or(0);
                println!("  \x1b[32m✓\x1b[0m {}: exists ({} note(s))", ref_path, note_count);
            }
            _ => {
                println!(
                    "  \x1b[33m⚠\x1b[0m {}: not initialized yet (run: git-notes sync pull)",
                    ref_path
                );
                warnings += 1;
            }
        }
    }

    // 4. Remote Fetch Refspec Check
    let remote_config = Command::new("git")
        .args(["config", "--get-all", "remote.origin.fetch"])
        .output();

    let has_notes_fetch = match remote_config {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).contains("refs/notes/*")
        }
        _ => false,
    };

    if has_notes_fetch {
        println!("  \x1b[32m✓\x1b[0m Remote fetch refspec: configured for refs/notes/*");
    } else {
        println!("  \x1b[33m⚠\x1b[0m Remote fetch refspec: missing '+refs/notes/*:refs/notes/*'");
        println!("    \x1b[90mFix with: git config --add remote.origin.fetch '+refs/notes/*:refs/notes/*'\x1b[0m");
        warnings += 1;
    }

    // 5. Hooks Status Check
    if let Some(ref root) = git_root {
        let hooks_dir = Path::new(root).join(".git").join("hooks");
        let post_merge = hooks_dir.join("post-merge");
        let pre_push = hooks_dir.join("pre-push");

        if post_merge.exists() {
            println!("  \x1b[32m✓\x1b[0m post-merge hook: installed");
        } else {
            println!("  \x1b[33m⚠\x1b[0m post-merge hook: not installed (run: git-notes-hooks install)");
            warnings += 1;
        }

        if pre_push.exists() {
            println!("  \x1b[32m✓\x1b[0m pre-push hook: installed");
        } else {
            println!("  \x1b[33m⚠\x1b[0m pre-push hook: not installed");
            warnings += 1;
        }

        // 6. Stale Notes Check
        let mut stale_count = 0;
        let notes_out = Command::new("git")
            .args(["notes", "--ref", "refs/notes/comments", "list"])
            .output();

        if let Ok(out) = notes_out {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if let Some(blob) = line.split_whitespace().next() {
                    if let Ok(cat) = Command::new("git").args(["cat-file", "blob", blob]).output() {
                        if let Ok(note) = serde_json::from_slice::<serde_json::Value>(&cat.stdout) {
                            if let Some(rel_file) = note.get("file").and_then(|f| f.as_str()) {
                                if !rel_file.is_empty() && !Path::new(root).join(rel_file).exists() {
                                    stale_count += 1;
                                    println!(
                                        "  \x1b[33m⚠\x1b[0m Stale note {}: referenced file '{}' no longer exists",
                                        &blob[..8.min(blob.len())],
                                        rel_file
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        if stale_count == 0 {
            println!("  \x1b[32m✓\x1b[0m Stale notes: all referenced files exist in working tree");
        } else {
            warnings += stale_count;
        }
    }

    // Summary
    println!();
    if errors == 0 && warnings == 0 {
        println!("\x1b[32m✨ All checks passed! git-notes is in peak health.\x1b[0m\n");
    } else {
        println!(
            "\x1b[1mSummary: \x1b[31m{} error(s)\x1b[0m, \x1b[33m{} warning(s)\x1b[0m\n",
            errors, warnings
        );
    }

    Ok(())
}
