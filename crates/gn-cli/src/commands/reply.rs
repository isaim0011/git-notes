use anyhow::{anyhow, Context, Result};
use clap::Args;
use gn_core::{Note, NotesEngine};
use std::process::Command;

#[derive(Args)]
pub struct ReplyArgs {
    /// Note ID, index number (1, 2, ...), or "latest" / "^" (defaults to latest)
    #[arg(default_value = "latest")]
    pub id: String,

    /// Reply message
    #[arg(short, long)]
    pub message: String,
}

pub fn run(args: &ReplyArgs) -> Result<()> {
    let engine = NotesEngine::new(".");
    let namespaces = vec!["comments", "review", "todos"];

    let mut all_notes = Vec::new();

    for ns in &namespaces {
        let namespace_enum = gn_core::Namespace::Custom(ns.to_string());
        if let Ok(notes) = engine.read_notes(&namespace_enum) {
            all_notes.extend(notes);
        }
    }

    if all_notes.is_empty() {
        return Err(anyhow!("No notes exist in the repository to reply to."));
    }

    let target = args.id.trim().trim_start_matches('#');
    let found_note = if target.eq_ignore_ascii_case("latest") || target == "^" {
        all_notes.last().cloned()
    } else if let Ok(idx) = target.parse::<usize>() {
        if idx >= 1 && idx <= all_notes.len() {
            Some(all_notes[idx - 1].clone())
        } else {
            None
        }
    } else {
        all_notes
            .iter()
            .find(|n| n.id.to_string().starts_with(target))
            .cloned()
    };

    let parent_note =
        found_note.ok_or_else(|| anyhow!("Target note '{}' not found", args.id))?;

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

    let reply = Note::reply(&parent_note, args.message.clone(), author);

    let id = engine.write_note(&reply)?;
    println!(
        "\x1b[32m✔\x1b[0m Reply added to thread \x1b[36m{}\x1b[0m (Note ID: \x1b[36m{}\x1b[0m)",
        &parent_note.id.to_string()[..8],
        &id[..8.min(id.len())]
    );

    Ok(())
}
