use anyhow::{Context, Result};
use clap::Args;
use gn_core::namespace::Namespace;
use gn_core::note::{Note, NoteStatus};
use gn_core::NotesEngine;
use std::env;
use std::process::Command;

#[derive(Args, Debug)]
pub struct ImportPrArgs {
    /// Pull Request number
    #[arg(short, long)]
    pub pr: u64,

    /// GitHub Personal Access Token (defaults to GITHUB_TOKEN env var)
    #[arg(short, long)]
    pub token: Option<String>,

    /// Namespace to store imported comments (default: review)
    #[arg(short, long, default_value = "review")]
    pub namespace: String,
}

pub fn run(args: &ImportPrArgs) -> Result<()> {
    let token = args
        .token
        .clone()
        .or_else(|| env::var("GITHUB_TOKEN").ok())
        .context("GitHub token required. Provide via --token or GITHUB_TOKEN env var.")?;

    // Detect owner/repo from git remote
    let remote_out = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to get origin remote URL")?;

    let remote_str = String::from_utf8_lossy(&remote_out.stdout).trim().to_string();
    let (owner, repo) = parse_github_owner_repo(&remote_str)
        .context("Could not determine GitHub owner/repo from remote URL")?;

    println!("Importing review comments from {}/{} PR #{}...", owner, repo, args.pr);

    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/comments",
        owner, repo, args.pr
    );

    let curl_output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            &format!("Authorization: token {}", token),
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: git-notes-cli",
            &url,
        ])
        .output()
        .context("Failed to run curl to fetch PR comments")?;

    if !curl_output.status.success() {
        let err = String::from_utf8_lossy(&curl_output.stderr);
        anyhow::bail!("GitHub API request failed: {}", err);
    }

    let comments: Vec<serde_json::Value> = serde_json::from_slice(&curl_output.stdout)
        .context("Failed to parse GitHub comments JSON")?;

    let engine = NotesEngine::new(".");
    let target_commit = match Command::new("git").args(["rev-parse", "HEAD"]).output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(_) => "HEAD".to_string(),
    };

    let mut imported = 0;
    let ns = Namespace::from_str(&args.namespace);

    for c in &comments {
        let body = match c.get("body").and_then(|b| b.as_str()) {
            Some(b) => b.to_string(),
            None => continue,
        };

        let file = c.get("path").and_then(|p| p.as_str()).map(|s| s.to_string());
        let line = c.get("line").and_then(|l| l.as_u64()).map(|l| l as u32);
        let author = c
            .get("user")
            .and_then(|u| u.get("login"))
            .and_then(|l| l.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut note = Note::new(
            target_commit.clone(),
            file,
            line,
            line,
            body,
            author,
            ns.clone(),
        );
        note.status = NoteStatus::Open;

        if let Ok(_) = engine.write_note(&note) {
            imported += 1;
        }
    }

    println!(
        "\x1b[32m✔\x1b[0m Successfully imported {} review comment(s) into refs/notes/{}",
        imported, args.namespace
    );

    Ok(())
}

fn parse_github_owner_repo(remote: &str) -> Option<(String, String)> {
    let clean = remote
        .trim_end_matches(".git")
        .replace("git@github.com:", "")
        .replace("https://github.com/", "")
        .replace("http://github.com/", "");

    let parts: Vec<&str> = clean.split('/').collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}
