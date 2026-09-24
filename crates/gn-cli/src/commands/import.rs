use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use gn_core::namespace::Namespace;
use gn_core::note::{Note, NoteStatus};
use gn_core::NotesEngine;
use std::env;
use std::process::Command;

#[derive(Args, Debug)]
pub struct ImportArgs {
    #[command(subcommand)]
    pub command: ImportCommands,
}

#[derive(Subcommand, Debug)]
pub enum ImportCommands {
    /// Import review comments from a GitHub Pull Request [wrap/delegate]
    Pr(super::import_pr::ImportPrArgs),

    /// Import discussion notes and comments from a GitLab Merge Request
    Gitlab(GitlabImportArgs),

    /// Import comments and discussions from a Bitbucket Cloud Pull Request
    Bitbucket(BitbucketImportArgs),

    /// Import issue comments from a Jira instance
    Jira(JiraImportArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GitlabImportArgs {
    /// Merge Request IID
    #[arg(short, long)]
    pub mr: u64,

    /// GitLab Project ID or URL-encoded path (e.g. group/repo or numeric ID)
    #[arg(short, long)]
    pub project_id: Option<String>,

    /// GitLab Personal Access Token / Bearer token (defaults to GITLAB_TOKEN env var)
    #[arg(short, long)]
    pub token: Option<String>,

    /// GitLab instance host (default: gitlab.com)
    #[arg(long, default_value = "gitlab.com")]
    pub host: Option<String>,

    /// Namespace to store imported comments (default: review)
    #[arg(short, long, default_value = "review")]
    pub namespace: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct BitbucketImportArgs {
    /// Bitbucket Pull Request ID
    #[arg(short, long)]
    pub pr: u64,

    /// Repository slug
    #[arg(short, long)]
    pub repo: Option<String>,

    /// Bitbucket workspace
    #[arg(short, long)]
    pub workspace: Option<String>,

    /// Bitbucket App Password / Access Token (defaults to BITBUCKET_TOKEN env var)
    #[arg(short, long)]
    pub token: Option<String>,

    /// Namespace to store imported comments (default: review)
    #[arg(short, long, default_value = "review")]
    pub namespace: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct JiraImportArgs {
    /// Jira Issue ID or Key (e.g. PROJ-123)
    #[arg(short, long)]
    pub issue: String,

    /// Jira host URL (e.g. https://your-domain.atlassian.net)
    #[arg(long)]
    pub host: String,

    /// Jira API token (defaults to JIRA_TOKEN or JIRA_API_TOKEN env var)
    #[arg(short, long)]
    pub token: Option<String>,

    /// Namespace to store imported comments (default: review)
    #[arg(short, long, default_value = "review")]
    pub namespace: Option<String>,
}

pub fn run(args: &ImportArgs) -> Result<()> {
    match &args.command {
        ImportCommands::Pr(pr_args) => super::import_pr::run(pr_args),
        ImportCommands::Gitlab(gl_args) => run_gitlab(gl_args),
        ImportCommands::Bitbucket(bb_args) => run_bitbucket(bb_args),
        ImportCommands::Jira(jira_args) => run_jira(jira_args),
    }
}

// -----------------------------------------------------------------------------
// GitLab Import
// -----------------------------------------------------------------------------

pub fn run_gitlab(args: &GitlabImportArgs) -> Result<()> {
    let token = args
        .token
        .clone()
        .or_else(|| env::var("GITLAB_TOKEN").ok())
        .context("GitLab token required. Provide via --token or GITLAB_TOKEN env var.")?;

    let host = args.host.as_deref().unwrap_or("gitlab.com");
    let host = host.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');

    let project_id = match &args.project_id {
        Some(pid) => pid.clone(),
        None => {
            // Detect from git remote origin
            let remote_url = get_git_remote_url("origin")?;
            parse_gitlab_project(&remote_url, host)
                .context("Could not determine GitLab project from git remote. Please provide --project-id.")?
        }
    };

    let encoded_project_id = urlencoding(&project_id);
    let url = format!(
        "https://{}/api/v4/projects/{}/merge_requests/{}/discussions",
        host, encoded_project_id, args.mr
    );

    println!(
        "Importing review discussions from GitLab [{}] MR !{}...",
        project_id, args.mr
    );

    let curl_args = vec![
        "-fsSL".to_string(),
        "-H".to_string(),
        format!("PRIVATE-TOKEN: {}", token),
        "-H".to_string(),
        "Accept: application/json".to_string(),
        "-H".to_string(),
        "User-Agent: git-notes-cli".to_string(),
        url,
    ];

    let output = Command::new("curl")
        .args(&curl_args)
        .output()
        .context("Failed to execute curl command")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("GitLab API request failed: {}", err);
    }

    let discussions: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .context("Failed to parse GitLab discussions JSON")?;

    let engine = NotesEngine::new(".");
    let target_commit = get_head_commit();
    let namespace_name = args.namespace.as_deref().unwrap_or("review");
    let ns = Namespace::from_str(namespace_name);

    let mut imported = 0;

    for discussion in &discussions {
        let notes_array = match discussion.get("notes").and_then(|n| n.as_array()) {
            Some(arr) => arr,
            None => continue,
        };

        for gl_note in notes_array {
            // Check if note is system note (e.g. status changes), skip if system
            if gl_note.get("system").and_then(|s| s.as_bool()).unwrap_or(false) {
                continue;
            }

            let body = match gl_note.get("body").and_then(|b| b.as_str()) {
                Some(b) => b.to_string(),
                None => continue,
            };

            let position = gl_note.get("position");
            let file = position
                .and_then(|p| p.get("new_path").or_else(|| p.get("old_path")))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());

            let line = position
                .and_then(|p| p.get("new_line").or_else(|| p.get("old_line")))
                .and_then(|l| l.as_u64())
                .map(|l| l as u32);

            let commit_sha = position
                .and_then(|p| p.get("head_sha"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| target_commit.clone());

            let author_name = gl_note
                .get("author")
                .and_then(|a| a.get("name").or_else(|| a.get("username")))
                .and_then(|n| n.as_str())
                .unwrap_or("unknown")
                .to_string();

            let mut note = Note::new(
                commit_sha,
                file,
                line,
                line,
                body,
                author_name,
                ns.clone(),
            );

            let resolved = gl_note.get("resolved").and_then(|r| r.as_bool()).unwrap_or(false);
            note.status = if resolved {
                NoteStatus::Resolved
            } else {
                NoteStatus::Open
            };

            if engine.write_note(&note).is_ok() {
                imported += 1;
            }
        }
    }

    println!(
        "\x1b[32m✔\x1b[0m Successfully imported {} GitLab discussion comment(s) into refs/notes/{}",
        imported, namespace_name
    );

    Ok(())
}

// -----------------------------------------------------------------------------
// Bitbucket Import
// -----------------------------------------------------------------------------

pub fn run_bitbucket(args: &BitbucketImportArgs) -> Result<()> {
    let token = args
        .token
        .clone()
        .or_else(|| env::var("BITBUCKET_TOKEN").ok())
        .context("Bitbucket token required. Provide via --token or BITBUCKET_TOKEN env var.")?;

    let (workspace, repo) = match (&args.workspace, &args.repo) {
        (Some(w), Some(r)) => (w.clone(), r.clone()),
        _ => {
            let remote_url = get_git_remote_url("origin")?;
            let (w, r) = parse_bitbucket_workspace_repo(&remote_url)
                .context("Could not determine Bitbucket workspace/repo from git remote. Please provide --workspace and --repo.")?;
            (args.workspace.clone().unwrap_or(w), args.repo.clone().unwrap_or(r))
        }
    };

    println!(
        "Importing review comments from Bitbucket {}/{} PR #{}...",
        workspace, repo, args.pr
    );

    let url = format!(
        "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/comments",
        workspace, repo, args.pr
    );

    let auth_header = if token.contains(':') {
        // Basic auth user:app_password
        use base64_helper::base64_encode;
        format!("Basic {}", base64_encode(token.as_bytes()))
    } else {
        // Bearer token
        format!("Bearer {}", token)
    };

    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            &format!("Authorization: {}", auth_header),
            "-H",
            "Accept: application/json",
            "-H",
            "User-Agent: git-notes-cli",
            &url,
        ])
        .output()
        .context("Failed to execute curl command")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Bitbucket API request failed: {}", err);
    }

    let json_resp: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse Bitbucket comments JSON")?;

    let comments = json_resp
        .get("values")
        .and_then(|v| v.as_array())
        .context("Bitbucket response missing 'values' array")?;

    let engine = NotesEngine::new(".");
    let target_commit = get_head_commit();
    let namespace_name = args.namespace.as_deref().unwrap_or("review");
    let ns = Namespace::from_str(namespace_name);

    let mut imported = 0;

    for c in comments {
        if c.get("deleted").and_then(|d| d.as_bool()).unwrap_or(false) {
            continue;
        }

        let body = match c.get("content").and_then(|cnt| cnt.get("raw")).and_then(|r| r.as_str()) {
            Some(b) => b.to_string(),
            None => continue,
        };

        let inline = c.get("inline");
        let file = inline
            .and_then(|i| i.get("path"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());

        let line = inline
            .and_then(|i| i.get("to").or_else(|| i.get("from")))
            .and_then(|l| l.as_u64())
            .map(|l| l as u32);

        let author_name = c
            .get("user")
            .and_then(|u| u.get("display_name").or_else(|| u.get("nickname")).or_else(|| u.get("account_id")))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut note = Note::new(
            target_commit.clone(),
            file,
            line,
            line,
            body,
            author_name,
            ns.clone(),
        );
        note.status = NoteStatus::Open;

        if engine.write_note(&note).is_ok() {
            imported += 1;
        }
    }

    println!(
        "\x1b[32m✔\x1b[0m Successfully imported {} Bitbucket comment(s) into refs/notes/{}",
        imported, namespace_name
    );

    Ok(())
}

// -----------------------------------------------------------------------------
// Jira Import
// -----------------------------------------------------------------------------

pub fn run_jira(args: &JiraImportArgs) -> Result<()> {
    let token = args
        .token
        .clone()
        .or_else(|| env::var("JIRA_TOKEN").ok())
        .or_else(|| env::var("JIRA_API_TOKEN").ok())
        .context("Jira API token required. Provide via --token or JIRA_TOKEN / JIRA_API_TOKEN env var.")?;

    let host = args.host.trim_end_matches('/');
    let host_url = if !host.starts_with("http://") && !host.starts_with("https://") {
        format!("https://{}", host)
    } else {
        host.to_string()
    };

    println!(
        "Importing comments from Jira issue [{}] at {}...",
        args.issue, host_url
    );

    let url = format!("{}/rest/api/3/issue/{}/comment", host_url, args.issue);

    let auth_header = if token.contains(':') {
        use base64_helper::base64_encode;
        format!("Basic {}", base64_encode(token.as_bytes()))
    } else if let Ok(email) = env::var("JIRA_EMAIL") {
        use base64_helper::base64_encode;
        format!("Basic {}", base64_encode(format!("{}:{}", email, token).as_bytes()))
    } else {
        format!("Bearer {}", token)
    };

    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            &format!("Authorization: {}", auth_header),
            "-H",
            "Accept: application/json",
            "-H",
            "User-Agent: git-notes-cli",
            &url,
        ])
        .output()
        .context("Failed to execute curl command")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Jira API request failed: {}", err);
    }

    let json_resp: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse Jira comments JSON")?;

    let comments = json_resp
        .get("comments")
        .and_then(|v| v.as_array())
        .context("Jira response missing 'comments' array")?;

    let engine = NotesEngine::new(".");
    let target_commit = get_head_commit();
    let namespace_name = args.namespace.as_deref().unwrap_or("review");
    let ns = Namespace::from_str(namespace_name);

    let mut imported = 0;

    for c in comments {
        let body_val = c.get("body");
        let body_str = extract_jira_comment_body(body_val);

        if body_str.trim().is_empty() {
            continue;
        }

        let body_with_prefix = format!("[Jira {}] {}", args.issue, body_str);

        let author_name = c
            .get("author")
            .and_then(|a| a.get("displayName").or_else(|| a.get("emailAddress")).or_else(|| a.get("accountId")))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut note = Note::new(
            target_commit.clone(),
            None,
            None,
            None,
            body_with_prefix,
            author_name,
            ns.clone(),
        );
        note.status = NoteStatus::Open;

        if engine.write_note(&note).is_ok() {
            imported += 1;
        }
    }

    println!(
        "\x1b[32m✔\x1b[0m Successfully imported {} Jira comment(s) into refs/notes/{}",
        imported, namespace_name
    );

    Ok(())
}

// -----------------------------------------------------------------------------
// Helpers & Parsers
// -----------------------------------------------------------------------------

fn get_head_commit() -> String {
    match Command::new("git").args(["rev-parse", "HEAD"]).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "HEAD".to_string(),
    }
}

fn get_git_remote_url(remote: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["remote", "get-url", remote])
        .output()
        .context("Failed to get remote URL")?;
    if !out.status.success() {
        anyhow::bail!("git remote get-url {} exited with non-zero status", remote);
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn parse_gitlab_project(remote: &str, host: &str) -> Option<String> {
    let clean = remote
        .trim_end_matches(".git")
        .replace(&format!("git@{}:", host), "")
        .replace(&format!("https://{}/", host), "")
        .replace(&format!("http://{}/", host), "");
    let clean = clean.trim_start_matches('/');
    if clean.is_empty() {
        None
    } else {
        Some(clean.to_string())
    }
}

pub fn parse_bitbucket_workspace_repo(remote: &str) -> Option<(String, String)> {
    let clean = remote
        .trim_end_matches(".git")
        .replace("git@bitbucket.org:", "")
        .replace("https://bitbucket.org/", "")
        .replace("http://bitbucket.org/", "");
    let clean = clean.trim_start_matches('/');
    let parts: Vec<&str> = clean.split('/').collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

fn urlencoding(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

fn extract_jira_comment_body(val: Option<&serde_json::Value>) -> String {
    let val = match val {
        Some(v) => v,
        None => return String::new(),
    };

    if let Some(s) = val.as_str() {
        return s.to_string();
    }

    // Atlassian Document Format (ADF) handling:
    // { "type": "doc", "content": [ { "type": "paragraph", "content": [ { "type": "text", "text": "..." } ] } ] }
    let mut text_acc = String::new();
    extract_adf_text(val, &mut text_acc);
    text_acc
}

fn extract_adf_text(val: &serde_json::Value, acc: &mut String) {
    if let Some(t) = val.get("text").and_then(|t| t.as_str()) {
        acc.push_str(t);
    }
    if let Some(content) = val.get("content").and_then(|c| c.as_array()) {
        for child in content {
            extract_adf_text(child, acc);
            if child.get("type").and_then(|t| t.as_str()) == Some("paragraph") {
                acc.push('\n');
            }
        }
    }
}

mod base64_helper {
    const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn base64_encode(input: &[u8]) -> String {
        let mut result = String::new();
        let len = input.len();
        let mut i = 0;

        while i < len {
            let b0 = input[i];
            let b1 = if i + 1 < len { input[i + 1] } else { 0 };
            let b2 = if i + 2 < len { input[i + 2] } else { 0 };

            result.push(B64_CHARS[((b0 >> 2) & 0x3F) as usize] as char);
            result.push(B64_CHARS[(((b0 & 0x03) << 4) | ((b1 >> 4) & 0x0F)) as usize] as char);

            if i + 1 < len {
                result.push(B64_CHARS[(((b1 & 0x0F) << 2) | ((b2 >> 6) & 0x03)) as usize] as char);
            } else {
                result.push('=');
            }

            if i + 2 < len {
                result.push(B64_CHARS[(b2 & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }

            i += 3;
        }

        result
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b"user:pass"), "dXNlcjpwYXNz");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gitlab_project() {
        assert_eq!(
            parse_gitlab_project("git@gitlab.com:group/subgroup/repo.git", "gitlab.com"),
            Some("group/subgroup/repo".to_string())
        );
        assert_eq!(
            parse_gitlab_project("https://gitlab.com/owner/project.git", "gitlab.com"),
            Some("owner/project".to_string())
        );
        assert_eq!(
            parse_gitlab_project("https://gitlab.example.org/myorg/myproject", "gitlab.example.org"),
            Some("myorg/myproject".to_string())
        );
    }

    #[test]
    fn test_parse_bitbucket_workspace_repo() {
        assert_eq!(
            parse_bitbucket_workspace_repo("git@bitbucket.org:myws/myrepo.git"),
            Some(("myws".to_string(), "myrepo".to_string()))
        );
        assert_eq!(
            parse_bitbucket_workspace_repo("https://bitbucket.org/team/project"),
            Some(("team".to_string(), "project".to_string()))
        );
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("group/subgroup/repo"), "group%2Fsubgroup%2Frepo");
        assert_eq!(urlencoding("12345"), "12345");
    }

    #[test]
    fn test_extract_jira_adf() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        { "type": "text", "text": "This is a Jira " },
                        { "type": "text", "text": "comment." }
                    ]
                }
            ]
        });
        let body = extract_jira_comment_body(Some(&adf));
        assert!(body.contains("This is a Jira comment."));
    }
}
