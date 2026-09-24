use anyhow::{Context, Result};
use clap::Args;
use gn_core::{Namespace, Note, NotesEngine};
use serde::Deserialize;
use std::io::{self, Read};
use std::process::Command;
use uuid::Uuid;

#[derive(Args)]
pub struct AddBulkArgs {
    /// JSON input string, file path, or '-' for stdin
    #[arg(short, long)]
    pub json: Option<String>,
}

#[derive(Deserialize)]
pub struct BulkNoteInput {
    pub id: Option<String>,
    pub commit: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub body: String,
    pub author: Option<String>,
    pub namespace: Option<String>,
    pub thread_id: Option<Uuid>,
}

pub fn run(args: &AddBulkArgs) -> Result<()> {
    let raw_json = match &args.json {
        Some(s) if s != "-" => {
            if std::path::Path::new(s).exists() {
                std::fs::read_to_string(s).context("Failed to read JSON file")?
            } else {
                s.clone()
            }
        }
        _ => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read stdin")?;
            buffer
        }
    };

    if raw_json.trim().is_empty() {
        return Ok(());
    }

    let inputs: Vec<BulkNoteInput> =
        serde_json::from_str(&raw_json).context("Failed to parse JSON input for bulk add")?;

    if inputs.is_empty() {
        return Ok(());
    }

    let name_output = Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok();
    let email_output = Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok();
    let default_author = match (name_output, email_output) {
        (Some(n), Some(e)) => format!(
            "{} <{}>",
            String::from_utf8_lossy(&n.stdout).trim(),
            String::from_utf8_lossy(&e.stdout).trim()
        ),
        _ => "Unknown <unknown@example.com>".to_string(),
    };

    let head_output = Command::new("git").args(["rev-parse", "HEAD"]).output().ok();
    let default_commit = head_output
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let notes: Vec<Note> = inputs
        .into_iter()
        .map(|i| {
            let commit = i.commit.unwrap_or_else(|| default_commit.clone());
            let author = i.author.unwrap_or_else(|| default_author.clone());
            let line_start = i.line_start.or(i.line);
            let line_end = i.line_end.or(line_start);
            let namespace_str = i.namespace.unwrap_or_else(|| "comments".to_string());
            let namespace = Namespace::from_str(&namespace_str);

            let mut note = Note::new(
                commit, i.file, line_start, line_end, i.body, author, namespace,
            );

            if let Some(id_str) = i.id {
                if !id_str.trim().is_empty() {
                    if let Ok(parsed_id) = Uuid::parse_str(&id_str) {
                        note.id = parsed_id;
                    }
                }
            }
            if let Some(tid) = i.thread_id {
                note.thread_id = Some(tid);
            }
            note
        })
        .collect();

    let engine = NotesEngine::new(".");
    engine.write_notes(&notes)?;

    println!("✓ Added {} notes in batch", notes.len());

    Ok(())
}
