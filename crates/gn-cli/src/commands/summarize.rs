use anyhow::{Context, Result};
use clap::Args;
use std::env;
use std::process::Command;

#[derive(Args, Debug)]
pub struct SummarizeArgs {
    /// Namespace to summarize (comments, review, todos, or all)
    #[arg(short, long, default_value = "all")]
    pub namespace: String,

    /// Gemini API key (or set GEMINI_API_KEY env var)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Only include open notes (default: true)
    #[arg(long, default_value = "true")]
    pub open_only: bool,

    /// Output format: text or markdown
    #[arg(long, default_value = "markdown")]
    pub format: String,
}

pub fn run(args: &SummarizeArgs) -> Result<()> {
    let api_key = args
        .api_key
        .clone()
        .or_else(|| env::var("GEMINI_API_KEY").ok())
        .context("No Gemini API key found. Set GEMINI_API_KEY or pass --api-key <key>")?;

    let repo_path = std::path::Path::new(".");

    // Collect notes from requested namespaces
    let namespaces: Vec<&str> = if args.namespace == "all" {
        vec!["comments", "review", "todos"]
    } else {
        vec![args.namespace.as_str()]
    };

    let mut all_notes_text = String::new();
    let mut total_count = 0;

    for ns in &namespaces {
        let output = Command::new("git")
            .args(["notes", "--ref", &format!("refs/notes/{}", ns), "list"])
            .current_dir(repo_path)
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let list = String::from_utf8_lossy(&out.stdout);
                for line in list.lines() {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() < 2 {
                        continue;
                    }
                    let note_blob = parts[0];
                    let commit = parts[1];

                    let content = Command::new("git")
                        .args(["cat-file", "blob", note_blob])
                        .current_dir(repo_path)
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .unwrap_or_default();

                    if content.trim().is_empty() {
                        continue;
                    }

                    // Try to parse as JSON Note, fall back to raw text
                    if let Ok(note) = serde_json::from_str::<serde_json::Value>(&content) {
                        let status = note["status"].as_str().unwrap_or("Open");
                        if args.open_only && status != "Open" {
                            continue;
                        }

                        let body = note["body"].as_str().unwrap_or("").trim().to_string();
                        let author = note["author"].as_str().unwrap_or("Unknown");
                        let file = note["file"].as_str().unwrap_or("");
                        let line = note["line_start"].as_u64().unwrap_or(0);
                        let note_id = note["id"].as_str().unwrap_or(&note_blob[..8.min(note_blob.len())]);

                        all_notes_text.push_str(&format!(
                            "- [{}] [{}/{}:{}] {} — by {} (status: {})\n",
                            &note_id[..8.min(note_id.len())],
                            ns, file, line,
                            body,
                            author,
                            status
                        ));
                        total_count += 1;
                    } else {
                        // Raw note content
                        all_notes_text.push_str(&format!(
                            "- [{}] [{}] {} (commit: {})\n",
                            &note_blob[..8.min(note_blob.len())],
                            ns,
                            content.lines().next().unwrap_or("").trim(),
                            &commit[..8.min(commit.len())]
                        ));
                        total_count += 1;
                    }
                }
            }
        }
    }

    if total_count == 0 {
        println!("No open notes found in namespace(s): {}", args.namespace);
        println!("Try: git-notes sync pull  to fetch notes from remote");
        return Ok(());
    }

    eprintln!("Summarizing {} note(s) via Gemini API...", total_count);

    // Build prompt
    let prompt = format!(
        r#"You are a senior engineering lead reviewing a codebase before a release.

Here are the open discussion threads and code annotations from the git-notes system:

{}

Please provide:
1. **Executive Summary** — 2-3 sentences: what is the overall health of the open discussions?
2. **Critical Issues** — Any notes that look like blockers, security concerns, or bugs (if none, say "None identified").
3. **Open Discussions** — Group and summarize the active conversations by theme or file area.
4. **Recommended Actions** — Specific next steps for the team, ordered by priority.
5. **Quick Stats** — Total open: {}, breakdown by namespace if multiple.

Be concise, technical, and actionable. Use markdown formatting."#,
        all_notes_text,
        total_count
    );

    let summary = call_gemini_api(&api_key, &prompt)?;

    println!("\n{}\n", summary);

    Ok(())
}

fn call_gemini_api(api_key: &str, prompt: &str) -> Result<String> {
    let model = "gemini-2.0-flash";
    let host = "generativelanguage.googleapis.com";
    let path = format!(
        "/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let body = serde_json::json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }],
        "generationConfig": {
            "temperature": 0.3,
            "maxOutputTokens": 2048
        }
    });

    let body_str = serde_json::to_string(&body)?;

    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-X", "POST",
            "-H", "Content-Type: application/json",
            "-d", &body_str,
            &format!("https://{}{}", host, path),
        ])
        .output()
        .context("curl not found — install curl to use git-notes summarize")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Gemini API request failed: {}", err);
    }

    let resp: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse Gemini API response")?;

    let text = resp["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .context("Unexpected Gemini API response format")?
        .to_string();

    Ok(text)
}
