use anyhow::{bail, Context, Result};
use clap::Args;
use gn_core::note::NoteStatus;
use gn_core::{Namespace, Note, NotesEngine};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Args, Debug, Clone)]
pub struct CheckArgs {
    /// Require at least N Approved notes on target commit/branch
    #[arg(long, default_value_t = 1)]
    pub min_approvals: usize,

    /// Fail if ANY note on current commit/branch is in Open status
    #[arg(long)]
    pub no_unresolved: bool,

    /// Check a specific namespace or all namespaces
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// Check a specific commit or default to HEAD
    #[arg(short, long)]
    pub commit: Option<String>,

    /// Output check results as structured JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct CheckSummary {
    pub commit: String,
    pub passed: bool,
    pub total_notes: usize,
    pub open_count: usize,
    pub approved_count: usize,
    pub rejected_count: usize,
    pub resolved_count: usize,
    pub min_approvals: usize,
    pub no_unresolved: bool,
    pub blocking_notes: Vec<BlockingNoteInfo>,
    pub failure_reasons: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockingNoteInfo {
    pub id: String,
    pub file: Option<String>,
    pub line_start: Option<u32>,
    pub author: String,
    pub status: String,
    pub body: String,
    pub reason: String,
}

/// Resolves the target commit SHA (defaults to HEAD via `git rev-parse HEAD`)
pub fn resolve_commit(repo_path: &Path, commit_arg: Option<&str>) -> Result<String> {
    let target = commit_arg.unwrap_or("HEAD");
    let output = Command::new("git")
        .args(["rev-parse", target])
        .current_dir(repo_path)
        .output()
        .with_context(|| format!("Failed to execute 'git rev-parse {}'", target))?;

    if !output.status.success() {
        bail!("Failed to resolve commit '{}': git rev-parse returned non-zero exit code", target);
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() {
        bail!("Failed to resolve commit '{}': output was empty", target);
    }
    Ok(sha)
}

/// Retrieves commits on the branch/history up to target commit if needed, or matches exact commit
pub fn get_ancestor_commits(repo_path: &Path, target_commit: &str) -> Vec<String> {
    if let Ok(output) = Command::new("git")
        .args(["rev-list", "-n", "100", target_commit])
        .current_dir(repo_path)
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    vec![target_commit.to_string()]
}

pub fn run(args: &CheckArgs) -> Result<()> {
    run_in_repo(Path::new("."), args)
}

pub fn run_in_repo(repo_path: &Path, args: &CheckArgs) -> Result<()> {
    let target_sha = resolve_commit(repo_path, args.commit.as_deref())?;
    let engine = NotesEngine::new(repo_path);

    let namespaces = match &args.namespace {
        Some(ns) => vec![Namespace::from_str(ns)],
        None => vec![
            Namespace::Comments,
            Namespace::Review,
            Namespace::Todos,
        ],
    };

    let mut all_notes: Vec<Note> = Vec::new();
    for ns in &namespaces {
        if let Ok(notes) = engine.read_notes(ns) {
            all_notes.extend(notes);
        }
    }

    let ancestors = get_ancestor_commits(repo_path, &target_sha);

    // Filter notes anchored to the target commit or its branch ancestors (prioritizing target commit)
    all_notes.retain(|n| n.commit == target_sha || ancestors.contains(&n.commit));

    let mut total_notes = 0;
    let mut open_notes: Vec<&Note> = Vec::new();
    let mut approved_notes: Vec<&Note> = Vec::new();
    let mut rejected_notes: Vec<&Note> = Vec::new();
    let mut resolved_notes: Vec<&Note> = Vec::new();

    for note in &all_notes {
        total_notes += 1;
        match note.status {
            NoteStatus::Open => open_notes.push(note),
            NoteStatus::Approved => approved_notes.push(note),
            NoteStatus::Rejected => rejected_notes.push(note),
            NoteStatus::Resolved => resolved_notes.push(note),
        }
    }

    let mut failure_reasons: Vec<String> = Vec::new();
    let mut blocking_notes: Vec<BlockingNoteInfo> = Vec::new();

    // 1. Rejected notes always block
    if !rejected_notes.is_empty() {
        failure_reasons.push(format!(
            "Found {} rejected note(s) requiring changes",
            rejected_notes.len()
        ));
        for n in &rejected_notes {
            blocking_notes.push(BlockingNoteInfo {
                id: n.id.to_string(),
                file: n.file.clone(),
                line_start: n.line_start,
                author: n.author.clone(),
                status: "Rejected".to_string(),
                body: n.body.clone(),
                reason: "Note is marked as Rejected".to_string(),
            });
        }
    }

    // 2. Open notes block if --no-unresolved is set
    if args.no_unresolved && !open_notes.is_empty() {
        failure_reasons.push(format!(
            "--no-unresolved specified and {} unresolved open note(s) remain",
            open_notes.len()
        ));
        for n in &open_notes {
            blocking_notes.push(BlockingNoteInfo {
                id: n.id.to_string(),
                file: n.file.clone(),
                line_start: n.line_start,
                author: n.author.clone(),
                status: "Open".to_string(),
                body: n.body.clone(),
                reason: "Unresolved note under --no-unresolved rule".to_string(),
            });
        }
    }

    // 3. Approval threshold check
    if approved_notes.len() < args.min_approvals {
        failure_reasons.push(format!(
            "Approval threshold not met: required {} approval(s), but found {}",
            args.min_approvals,
            approved_notes.len()
        ));
    }

    let passed = failure_reasons.is_empty();

    let summary = CheckSummary {
        commit: target_sha.clone(),
        passed,
        total_notes,
        open_count: open_notes.len(),
        approved_count: approved_notes.len(),
        rejected_count: rejected_notes.len(),
        resolved_count: resolved_notes.len(),
        min_approvals: args.min_approvals,
        no_unresolved: args.no_unresolved,
        blocking_notes,
        failure_reasons: failure_reasons.clone(),
    };

    if args.json {
        let json_str = serde_json::to_string_pretty(&summary)?;
        println!("{}", json_str);
    } else if passed {
        println!(
            "\x1b[32m✔ Quality gate passed: {} notes reviewed ({} approved, 0 blocking)\x1b[0m",
            total_notes,
            approved_notes.len()
        );
    } else {
        let target_short = &target_sha[..target_sha.len().min(8)];
        eprintln!(
            "\n\x1b[1;31m✖ Quality gate failed for commit {}\x1b[0m",
            target_short
        );
        for reason in &failure_reasons {
            eprintln!("  \x1b[31m• {}\x1b[0m", reason);
        }

        if !summary.blocking_notes.is_empty() {
            eprintln!("\n\x1b[1;33mBlocking Notes:\x1b[0m");
            for bn in &summary.blocking_notes {
                let short_id = &bn.id[..bn.id.len().min(8)];
                let loc = match (&bn.file, bn.line_start) {
                    (Some(f), Some(l)) => format!("{}:{}", f, l),
                    (Some(f), None) => f.clone(),
                    (None, _) => "<global>".to_string(),
                };
                let author_short = bn.author.split('<').next().unwrap_or(&bn.author).trim();
                let body_short = bn.body.replace('\n', " ");
                let body_snippet = if body_short.len() > 60 {
                    format!("{}...", &body_short[..57])
                } else {
                    body_short
                };

                let status_color = if bn.status == "Rejected" {
                    "\x1b[31mRejected\x1b[0m"
                } else {
                    "\x1b[33mOpen\x1b[0m"
                };

                eprintln!(
                    "  [{}] {} ({}) by {} - \"{}\" [{}]",
                    short_id, loc, status_color, author_short, body_snippet, bn.reason
                );
            }
        }
        eprintln!();
    }

    if !passed {
        bail!("Quality gate failed: {}", failure_reasons.join("; "));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_git_repo(dir: &Path) {
        let run = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(dir)
                .output()
                .expect("git failed");
            assert!(out.status.success());
        };
        run(&["init"]);
        run(&["config", "user.email", "test@test.com"]);
        run(&["config", "user.name", "Test User"]);

        let file_path = dir.join("main.rs");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "fn main() {{}}").unwrap();
        run(&["add", "main.rs"]);
        run(&["commit", "-m", "Initial commit"]);
    }

    #[test]
    fn test_gate_pass_with_approval() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());

        let sha = resolve_commit(temp.path(), None).unwrap();
        let engine = NotesEngine::new(temp.path());

        let mut note = Note::new(
            sha.clone(),
            Some("main.rs".to_string()),
            Some(1),
            Some(1),
            "LGTM!".to_string(),
            "Reviewer <rev@example.com>".to_string(),
            Namespace::Comments,
        );
        note.status = NoteStatus::Approved;
        engine.write_note(&note).unwrap();

        let args = CheckArgs {
            min_approvals: 1,
            no_unresolved: true,
            namespace: None,
            commit: None,
            json: false,
        };

        let res = run_in_repo(temp.path(), &args);
        assert!(res.is_ok());
    }

    #[test]
    fn test_gate_fail_insufficient_approvals() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());

        let args = CheckArgs {
            min_approvals: 1,
            no_unresolved: false,
            namespace: None,
            commit: None,
            json: false,
        };

        let res = run_in_repo(temp.path(), &args);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Approval threshold not met"));
    }

    #[test]
    fn test_gate_fail_rejected_note() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());

        let sha = resolve_commit(temp.path(), None).unwrap();
        let engine = NotesEngine::new(temp.path());

        // Approved note
        let mut note1 = Note::new(
            sha.clone(),
            Some("main.rs".to_string()),
            Some(1),
            Some(1),
            "Approved note".to_string(),
            "Lead <lead@example.com>".to_string(),
            Namespace::Comments,
        );
        note1.status = NoteStatus::Approved;
        engine.write_note(&note1).unwrap();

        // Rejected note
        let mut note2 = Note::new(
            sha.clone(),
            Some("main.rs".to_string()),
            Some(1),
            Some(1),
            "Needs rework!".to_string(),
            "Senior <snr@example.com>".to_string(),
            Namespace::Comments,
        );
        note2.status = NoteStatus::Rejected;
        engine.write_note(&note2).unwrap();

        let args = CheckArgs {
            min_approvals: 1,
            no_unresolved: false,
            namespace: None,
            commit: None,
            json: false,
        };

        let res = run_in_repo(temp.path(), &args);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("rejected note"));
    }

    #[test]
    fn test_gate_fail_unresolved_open_notes() {
        let temp = TempDir::new().unwrap();
        setup_git_repo(temp.path());

        let sha = resolve_commit(temp.path(), None).unwrap();
        let engine = NotesEngine::new(temp.path());

        let mut note1 = Note::new(
            sha.clone(),
            Some("main.rs".to_string()),
            Some(1),
            Some(1),
            "Approved note".to_string(),
            "Lead <lead@example.com>".to_string(),
            Namespace::Comments,
        );
        note1.status = NoteStatus::Approved;
        engine.write_note(&note1).unwrap();

        let note2 = Note::new(
            sha.clone(),
            Some("main.rs".to_string()),
            Some(1),
            Some(1),
            "Questions about this".to_string(),
            "Junior <jr@example.com>".to_string(),
            Namespace::Comments,
        );
        // NoteStatus::Open by default
        engine.write_note(&note2).unwrap();

        let args = CheckArgs {
            min_approvals: 1,
            no_unresolved: true,
            namespace: None,
            commit: None,
            json: false,
        };

        let res = run_in_repo(temp.path(), &args);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("--no-unresolved"));
    }
}
