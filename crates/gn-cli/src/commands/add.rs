use clap::Args;
use gn_core::{NotesEngine, Note};
use std::process::Command;
use anyhow::{Context, Result};

#[derive(Args)]
pub struct AddArgs {
    /// File to attach the note to
    #[arg(short, long)]
    pub file: String,

    /// Line range (e.g., "10" or "10-15")
    #[arg(short, long)]
    pub line: String,

    /// Note message
    #[arg(short, long)]
    pub message: String,

    /// Namespace to add the note to
    #[arg(short, long, default_value = "comments")]
    pub namespace: String,

    /// Thread parent note ID (for replies)
    #[arg(short, long)]
    pub thread: Option<String>,
}

pub fn run(args: &AddArgs) -> Result<()> {
    let name_output = Command::new("git").args(["config", "user.name"]).output().context("Failed to read user.name")?;
    let email_output = Command::new("git").args(["config", "user.email"]).output().context("Failed to read user.email")?;
    
    let author = format!(
        "{} <{}>",
        String::from_utf8_lossy(&name_output.stdout).trim(),
        String::from_utf8_lossy(&email_output.stdout).trim()
    );

    let head_output = Command::new("git").args(["rev-parse", "HEAD"]).output().context("Failed to get HEAD commit")?;
    let commit = String::from_utf8_lossy(&head_output.stdout).trim().to_string();

    let mut line_start = None;
    let mut line_end = None;
    if !args.line.is_empty() {
        if args.line.contains('-') {
            let parts: Vec<&str> = args.line.split('-').collect();
            if parts.len() == 2 {
                line_start = parts[0].parse().ok();
                line_end = parts[1].parse().ok();
            }
        } else {
            line_start = args.line.parse().ok();
            line_end = line_start;
        }
    }

    let namespace = gn_core::Namespace::Custom(args.namespace.clone());

    let mut note = Note::new(
        commit,
        Some(args.file.clone()),
        line_start,
        line_end,
        args.message.clone(),
        author,
        namespace,
    );

    if let Some(thread_str) = &args.thread {
        if let Ok(tid) = uuid::Uuid::parse_str(thread_str) {
            note.thread_id = Some(tid);
        }
    }

    let engine = NotesEngine::new(".");
    engine.write_note(&note)?;

    println!("✓ Note {} added to refs/notes/{}", note.id, args.namespace);
    
    Ok(())
}
