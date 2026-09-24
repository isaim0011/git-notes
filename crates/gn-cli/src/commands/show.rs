use anyhow::{anyhow, Result};
use clap::Args;
use gn_core::NotesEngine;

#[derive(Args)]
pub struct ShowArgs {
    /// Note ID, index number (1, 2, ...), or "latest" / "^" (interactive picker if omitted)
    pub id: Option<String>,

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

    let mut all_notes = Vec::new();

    for ns in &namespaces {
        let namespace_enum = gn_core::Namespace::Custom(ns.to_string());
        if let Ok(notes) = engine.read_notes(&namespace_enum) {
            all_notes.extend(notes);
        }
    }

    if all_notes.is_empty() {
        println!("No notes found in repository.");
        return Ok(());
    }

    // If ID is omitted and in interactive terminal, prompt interactive picker!
    let found_note = match &args.id {
        None => {
            if args.json {
                all_notes.last().cloned()
            } else {
                match super::picker::pick_note("Select note to view:", &all_notes)? {
                    Some(n) => Some(n.clone()),
                    None => {
                        println!("Cancelled.");
                        return Ok(());
                    }
                }
            }
        }
        Some(raw_id) => {
            let target = raw_id.trim().trim_start_matches('#');
            if target.eq_ignore_ascii_case("latest") || target == "^" {
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
            }
        }
    };

    let note = found_note.ok_or_else(|| anyhow!("Note not found"))?;

    if args.json {
        if args.thread {
            let mut thread = vec![note.clone()];
            for n in &all_notes {
                if n.thread_id == Some(note.id) {
                    thread.push(n.clone());
                }
            }
            println!("{}", serde_json::to_string_pretty(&thread)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&note)?);
        }
    } else {
        println!("\x1b[1;36mNote {}\x1b[0m", note.id);
        println!("{}", "─".repeat(60));
        println!("Commit:     {}", note.commit);
        if let Some(f) = &note.file {
            let l = note.line_start.unwrap_or(0);
            println!("File:       {}:{}", f, l);
        }
        println!("Author:     {}", note.author);
        println!("Date:       {}", note.timestamp);
        println!("Namespace:  {:?}", note.namespace);
        println!("Status:     {:?}", note.status);
        println!("\nMessage:\n{}", note.body);

        if args.thread {
            let replies: Vec<&gn_core::note::Note> = all_notes
                .iter()
                .filter(|n| n.thread_id == Some(note.id))
                .collect();

            if !replies.is_empty() {
                println!("\n\x1b[1mThread Replies ({}):\x1b[0m", replies.len());
                println!("{}", "─".repeat(60));
                for r in replies {
                    println!("\x1b[36m↳ [{}]\x1b[0m {}:", &r.id.to_string()[..8], r.author);
                    println!("  {}\n", r.body.trim());
                }
            }
        }
    }

    Ok(())
}
