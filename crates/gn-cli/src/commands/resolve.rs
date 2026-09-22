use anyhow::{anyhow, Result};
use clap::Args;
use gn_core::{NoteStatus, NotesEngine};

#[derive(Args)]
pub struct ResolveArgs {
    /// Note ID prefix
    pub id: String,

    /// New status (approved, rejected, resolved, open)
    #[arg(short, long)]
    pub status: String,
}

pub fn run(args: &ResolveArgs) -> Result<()> {
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

    let mut note = found_note.ok_or_else(|| anyhow!("Note with ID {} not found", args.id))?;

    let status = match args.status.to_lowercase().as_str() {
        "approved" => NoteStatus::Approved,
        "rejected" => NoteStatus::Rejected,
        "resolved" => NoteStatus::Resolved,
        "open" => NoteStatus::Open,
        _ => {
            return Err(anyhow!(
                "Invalid status. Use approved, rejected, resolved, or open."
            ))
        }
    };

    note.status = status;

    engine.write_note(&note)?;

    println!(
        "✓ Note {} marked as {}",
        note.id,
        args.status.to_lowercase()
    );

    Ok(())
}
