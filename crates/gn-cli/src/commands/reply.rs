use anyhow::{anyhow, Context, Result};
use clap::Args;
use gn_core::{Note, NotesEngine};
use std::process::Command;

#[derive(Args)]
pub struct ReplyArgs {
    /// Parent note ID (or prefix) to reply to
    pub id: String,

    /// Reply message
    #[arg(short, long)]
    pub message: String,
}

pub fn run(args: &ReplyArgs) -> Result<()> {
    let engine = NotesEngine::new(".");
    let namespaces = vec!["comments", "review", "todos"];

    let mut found_note = None;

    for ns in &namespaces {
        let namespace_enum = gn_core::Namespace::Custom(ns.to_string());
        if let Ok(notes) = engine.read_notes(&namespace_enum) {
            for note in notes {
                if note.id.to_string().starts_with(&args.id) {
                    found_note = Some(note.clone());
                    break;
                }
            }
        }
        if found_note.is_some() {
            break;
        }
    }

    let parent_note =
        found_note.ok_or_else(|| anyhow!("Parent note with ID {} not found", args.id))?;

    // Get git author name and email
    let name_output = Command::new("git")
        .args(["config", "user.name"])
        .output()
        .context("Failed to read user.name")?;
    let email_output = Command::new("git")
        .args(["config", "user.email"])
        .output()
        .context("Failed to read user.email")?;

    let author = format!(
        "{} <{}>",
        String::from_utf8_lossy(&name_output.stdout).trim(),
        String::from_utf8_lossy(&email_output.stdout).trim()
    );

    let reply_note = Note::reply(&parent_note, args.message.clone(), author);

    engine.write_note(&reply_note)?;

    println!(
        "✓ Reply {} added to thread {} in namespace {}",
        reply_note.id, parent_note.id, parent_note.namespace
    );

    Ok(())
}
