use anyhow::{Context, Result};
use clap::Args;
use gn_core::{Namespace, Note, NotesEngine};
use std::process::Command;

#[derive(Args, Debug)]
pub struct RebaseHealArgs {
    /// Namespace to check notes from (default: all standard namespaces)
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// Dry run: detect and report re-anchoring candidates without applying changes
    #[arg(long)]
    pub dry_run: bool,

    /// Verbose logging of commit matching heuristics
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone)]
struct CommitMetadata {
    sha: String,
    subject: String,
    patch_id: Option<String>,
    changed_files: Vec<String>,
}

/// Execute git command and return trimmed stdout if successful
fn git_output(repo_path: &std::path::Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Check if a commit is an ancestor of HEAD
fn is_ancestor_of_head(repo_path: &std::path::Path, commit: &str) -> bool {
    let status = Command::new("git")
        .args(["merge-base", "--is-ancestor", commit, "HEAD"])
        .current_dir(repo_path)
        .status();
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

/// Verify if a commit object exists in git repo
fn verify_commit_exists(repo_path: &std::path::Path, commit: &str) -> bool {
    let arg = format!("{}^{{commit}}", commit);
    let status = Command::new("git")
        .args(["rev-parse", "--verify", "-q", &arg])
        .current_dir(repo_path)
        .status();
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

/// Get git patch-id for a given commit
fn get_patch_id(repo_path: &std::path::Path, commit: &str) -> Option<String> {
    use std::io::Write;
    use std::process::Stdio;

    let diff_output = Command::new("git")
        .args(["diff-tree", "-p", "--root", commit])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if !diff_output.status.success() || diff_output.stdout.is_empty() {
        return None;
    }

    let mut patch_id_cmd = Command::new("git")
        .args(["patch-id", "--stable"])
        .current_dir(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;

    if let Some(mut stdin) = patch_id_cmd.stdin.take() {
        let _ = stdin.write_all(&diff_output.stdout);
    }

    let output = patch_id_cmd.wait_with_output().ok()?;
    if output.status.success() {
        let out_str = String::from_utf8_lossy(&output.stdout);
        // git patch-id prints: "<patch_id> <commit_sha>"
        let parts: Vec<&str> = out_str.split_whitespace().collect();
        if !parts.is_empty() {
            return Some(parts[0].to_string());
        }
    }
    None
}

/// Retrieve the commit subject for a given commit SHA
fn get_commit_subject(repo_path: &std::path::Path, commit: &str) -> Option<String> {
    git_output(repo_path, &["log", "-1", "--format=%s", commit])
}

/// Retrieve changed files for a commit
fn get_commit_files(repo_path: &std::path::Path, commit: &str) -> Vec<String> {
    if let Some(out) = git_output(repo_path, &["diff-tree", "--no-commit-id", "--name-only", "-r", "--root", commit]) {
        out.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        Vec::new()
    }
}

/// Load recent commits on current HEAD branch (up to 100)
fn load_head_history(repo_path: &std::path::Path) -> Vec<CommitMetadata> {
    let log_out = match git_output(repo_path, &["log", "-n", "100", "--format=%H%x09%s"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let mut commits = Vec::new();
    for line in log_out.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.is_empty() {
            continue;
        }
        let sha = parts[0].to_string();
        let subject = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            String::new()
        };
        commits.push(CommitMetadata {
            sha,
            subject,
            patch_id: None,
            changed_files: Vec::new(),
        });
    }
    commits
}

/// Find the new commit corresponding to an old commit SHA
fn find_new_commit(
    repo_path: &std::path::Path,
    old_sha: &str,
    note: &Note,
    head_commits: &mut [CommitMetadata],
    verbose: bool,
) -> Option<String> {
    // 1. Try patch-id matching if old commit exists in repo objects
    if verify_commit_exists(repo_path, old_sha) {
        if let Some(old_patch_id) = get_patch_id(repo_path, old_sha) {
            if verbose {
                println!(
                    "  [heuristic:patch-id] old commit {} has patch-id {}",
                    &old_sha[..old_sha.len().min(8)],
                    &old_patch_id[..old_patch_id.len().min(8)]
                );
            }
            for candidate in head_commits.iter_mut() {
                if candidate.patch_id.is_none() {
                    candidate.patch_id = get_patch_id(repo_path, &candidate.sha);
                }
                if let Some(ref cand_patch_id) = candidate.patch_id {
                    if cand_patch_id == &old_patch_id {
                        if verbose {
                            println!(
                                "  [heuristic:patch-id] match found: candidate commit {}",
                                &candidate.sha[..candidate.sha.len().min(8)]
                            );
                        }
                        return Some(candidate.sha.clone());
                    }
                }
            }
        }
    }

    // 2. Try commit subject match if available
    let old_subject = get_commit_subject(repo_path, old_sha);
    if let Some(ref subj) = old_subject {
        let trimmed_subj = subj.trim();
        if !trimmed_subj.is_empty() {
            // Check exact subject match
            let matches: Vec<&CommitMetadata> = head_commits
                .iter()
                .filter(|c| c.subject.trim() == trimmed_subj)
                .collect();

            if matches.len() == 1 {
                if verbose {
                    println!(
                        "  [heuristic:subject] unique subject match: '{}' -> {}",
                        trimmed_subj,
                        &matches[0].sha[..matches[0].sha.len().min(8)]
                    );
                }
                return Some(matches[0].sha.clone());
            } else if matches.len() > 1 {
                // If multiple commits have the same subject, disambiguate using note.file
                if let Some(ref note_file) = note.file {
                    for m in &matches {
                        let files = get_commit_files(repo_path, &m.sha);
                        if files.iter().any(|f| f == note_file) {
                            if verbose {
                                println!(
                                    "  [heuristic:subject+file] disambiguated '{}' with file '{}' -> {}",
                                    trimmed_subj,
                                    note_file,
                                    &m.sha[..m.sha.len().min(8)]
                                );
                            }
                            return Some(m.sha.clone());
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: if note targets a specific file, check commits touching that file
    if let Some(ref note_file) = note.file {
        for candidate in head_commits.iter_mut() {
            if candidate.changed_files.is_empty() {
                candidate.changed_files = get_commit_files(repo_path, &candidate.sha);
            }
            if candidate.changed_files.iter().any(|f| f == note_file) {
                // If old subject was non-empty and starts with candidate subject or vice-versa
                if let Some(ref subj) = old_subject {
                    if !subj.is_empty()
                        && (subj.starts_with(&candidate.subject)
                            || candidate.subject.starts_with(subj))
                    {
                        if verbose {
                            println!(
                                "  [heuristic:file+partial_subject] matched file '{}' and subject prefix -> {}",
                                note_file,
                                &candidate.sha[..candidate.sha.len().min(8)]
                            );
                        }
                        return Some(candidate.sha.clone());
                    }
                }
            }
        }
    }

    None
}

pub fn run(args: &RebaseHealArgs) -> Result<()> {
    run_in_repo(std::path::Path::new("."), args)
}

pub fn run_in_repo(repo_path: &std::path::Path, args: &RebaseHealArgs) -> Result<()> {
    let engine = NotesEngine::new(repo_path);

    let namespaces = match &args.namespace {
        Some(ns) => vec![Namespace::from_str(ns)],
        None => vec![
            Namespace::Comments,
            Namespace::Review,
            Namespace::Todos,
        ],
    };

    let mut head_commits = load_head_history(repo_path);

    let mut healed_count = 0;
    let mut valid_count = 0;
    let mut unresolved_count = 0;

    for ns in &namespaces {
        let notes = match engine.read_notes(ns) {
            Ok(n) => n,
            Err(_) => continue,
        };

        for mut note in notes {
            let old_sha = &note.commit;
            let note_short_id = &note.id.to_string()[..8];
            let location = match (&note.file, note.line_start) {
                (Some(f), Some(l)) => format!("{}:{}", f, l),
                (Some(f), None) => f.clone(),
                (None, _) => "<no file>".to_string(),
            };

            // Check if note's commit is already on the current branch (HEAD history)
            if verify_commit_exists(repo_path, old_sha) && is_ancestor_of_head(repo_path, old_sha) {
                valid_count += 1;
                if args.verbose {
                    println!(
                        "Note {} anchor {} is already valid on HEAD.",
                        note_short_id,
                        &old_sha[..old_sha.len().min(8)]
                    );
                }
                continue;
            }

            // Note is anchored to an unreachable or rewritten commit
            if args.verbose {
                println!(
                    "Note {} anchor {} is not on HEAD. Attempting to heal...",
                    note_short_id,
                    &old_sha[..old_sha.len().min(8)]
                );
            }

            let found_commit = find_new_commit(repo_path, old_sha, &note, &mut head_commits, args.verbose);

            match found_commit {
                Some(new_sha) => {
                    let old_short = &old_sha[..old_sha.len().min(8)];
                    let new_short = &new_sha[..new_sha.len().min(8)];

                    if args.dry_run {
                        println!(
                            "[dry-run] Would re-anchor note {} from {} to {} ({})",
                            note_short_id, old_short, new_short, location
                        );
                    } else {
                        note.commit = new_sha.clone();
                        engine
                            .write_note(&note)
                            .with_context(|| format!("Failed to update note {}", note.id))?;
                        println!(
                            "✓ Re-anchored note {} to {} (file: {})",
                            note_short_id,
                            new_short,
                            note.file.as_deref().unwrap_or("<no file>")
                        );
                    }
                    healed_count += 1;
                }
                None => {
                    unresolved_count += 1;
                    if args.verbose {
                        println!(
                            "✗ Could not resolve new commit for note {} (old anchor: {})",
                            note_short_id,
                            &old_sha[..old_sha.len().min(8)]
                        );
                    }
                }
            }
        }
    }

    println!(
        "✨ Rebase healing complete: {} healed, {} already valid, {} unresolved.",
        healed_count, valid_count, unresolved_count
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn run_git(dir: &std::path::Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git cmd failed");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[test]
    fn test_rebase_heal_amended_commit() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        run_git(repo_path, &["init"]);
        run_git(repo_path, &["config", "user.email", "test@test.com"]);
        run_git(repo_path, &["config", "user.name", "Test User"]);

        // Commit 1: base
        let file_path = repo_path.join("file.txt");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Initial content").unwrap();
        run_git(repo_path, &["add", "file.txt"]);
        run_git(repo_path, &["commit", "-m", "Initial commit"]);

        // Commit 2: feature
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Feature content line 1\nFeature content line 2").unwrap();
        run_git(repo_path, &["add", "file.txt"]);
        run_git(repo_path, &["commit", "-m", "Add feature"]);
        let old_feature_sha = run_git(repo_path, &["rev-parse", "HEAD"]);

        // Create a note anchored to Commit 2
        let engine = NotesEngine::new(repo_path);
        let note = Note::new(
            old_feature_sha.clone(),
            Some("file.txt".to_string()),
            Some(1),
            Some(2),
            "Check this feature".to_string(),
            "Reviewer <rev@test.com>".to_string(),
            Namespace::Comments,
        );
        engine.write_note(&note).unwrap();

        // Now amend Commit 2 (changes SHA)
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Feature content line 1\nFeature content line 2 modified").unwrap();
        run_git(repo_path, &["add", "file.txt"]);
        run_git(repo_path, &["commit", "--amend", "-m", "Add feature"]);
        let new_feature_sha = run_git(repo_path, &["rev-parse", "HEAD"]);

        assert_ne!(old_feature_sha, new_feature_sha);

        // Run rebase heal on test repo
        let args = RebaseHealArgs {
            namespace: Some("comments".to_string()),
            dry_run: false,
            verbose: true,
        };

        let res = run_in_repo(repo_path, &args);
        assert!(res.is_ok());

        // Verify note was re-anchored to new_feature_sha
        let notes = engine.read_notes(&Namespace::Comments).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].commit, new_feature_sha);
    }

    #[test]
    fn test_rebase_heal_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        run_git(repo_path, &["init"]);
        run_git(repo_path, &["config", "user.email", "test@test.com"]);
        run_git(repo_path, &["config", "user.name", "Test User"]);

        let file_path = repo_path.join("file.txt");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Initial").unwrap();
        run_git(repo_path, &["add", "file.txt"]);
        run_git(repo_path, &["commit", "-m", "Initial commit"]);

        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Feature 1").unwrap();
        run_git(repo_path, &["add", "file.txt"]);
        run_git(repo_path, &["commit", "-m", "Feature 1"]);
        let old_feature_sha = run_git(repo_path, &["rev-parse", "HEAD"]);

        let engine = NotesEngine::new(repo_path);
        let note = Note::new(
            old_feature_sha.clone(),
            Some("file.txt".to_string()),
            Some(1),
            Some(1),
            "Needs check".to_string(),
            "Reviewer <rev@test.com>".to_string(),
            Namespace::Comments,
        );
        engine.write_note(&note).unwrap();

        // Amend commit
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Feature 1 amended").unwrap();
        run_git(repo_path, &["add", "file.txt"]);
        run_git(repo_path, &["commit", "--amend", "-m", "Feature 1"]);

        let dry_args = RebaseHealArgs {
            namespace: Some("comments".to_string()),
            dry_run: true,
            verbose: false,
        };

        let res = run_in_repo(repo_path, &dry_args);
        assert!(res.is_ok());

        // In dry run, note should still have old commit SHA
        let notes = engine.read_notes(&Namespace::Comments).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].commit, old_feature_sha);
    }
}
