use anyhow::{anyhow, Result};
use clap::Args;
use gn_core::NotesEngine;

#[derive(Args)]
pub struct ShowArgs {
    /// Note ID prefix
    pub id: String,

    /// Show entire thread
    #[arg(short, long)]
    pub thread: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &ShowArgs) -> Result<()> {
    let engine = NotesEngine::new(".");
    let namespaces = vec!["comments", "review", "todos"];

    let mut found_note = None;
    let mut all_notes = Vec::new();

    for ns in &namespaces {
        let namespace_enum = gn_core::Namespace::Custom(ns.to_string());
        if let Ok(notes) = engine.read_notes(&namespace_enum) {
            all_notes.extend(notes);
        }
    }

    for note in &all_notes {
        if note.id.to_string().starts_with(&args.id) {
            found_note = Some(note.clone());
            break;
        }
    }

    let note = found_note.ok_or_else(|| anyhow!("Note with ID {} not found", args.id))?;

    let replies: Vec<&gn_core::Note> = if args.thread {
        all_notes
            .iter()
            .filter(|n| n.thread_id.as_ref() == Some(&note.id))
            .collect()
    } else {
        Vec::new()
    };

    if args.json {
        if args.thread {
            let mut thread = Vec::with_capacity(1 + replies.len());
            thread.push(note.clone());
            thread.extend(replies.into_iter().cloned());
            println!("{}", serde_json::to_string_pretty(&thread)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&note)?);
        }
        return Ok(());
    }

    println!("ID: {}", note.id);
    println!("Commit: {}", note.commit);
    println!(
        "File: {}:{}",
        note.file.unwrap_or_default(),
        note.line_start.unwrap_or(0)
    );
    println!("Author: {}", note.author);
    println!("Date: {}", note.timestamp);
    println!("Status: {:?}", note.status);
    if let Some(parent) = &note.thread_id {
        println!("In-Reply-To: {}", parent);
    }
    println!("\n{}", note.body);

    if args.thread {
        for n in replies {
            println!("\n--- Reply: {} ---", n.id);
            println!("Author: {} | Date: {}", n.author, n.timestamp);
            println!("{}", n.body);
        }
    }

    Ok(())
}
