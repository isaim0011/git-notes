use anyhow::{anyhow, Result};
use clap::Args;
use gn_core::{NoteStatus, NotesEngine};

#[derive(Args)]
pub struct ResolveArgs {
    /// Note ID, index number (1, 2, ...), or "latest" / "^" (interactive picker if omitted)
    pub id: Option<String>,

    /// New status (approved, rejected, resolved, open)
    #[arg(short, long, default_value = "resolved")]
    pub status: String,
}

pub fn run(args: &ResolveArgs) -> Result<()> {
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
        return Err(anyhow!("No notes found in repository."));
    }

    // Interactive picker if ID omitted
    let found_note = match &args.id {
        None => match super::picker::pick_note("Select note to resolve:", &all_notes)? {
            Some(n) => Some(n.clone()),
            None => {
                println!("Cancelled.");
                return Ok(());
            }
        },
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

    let mut note = found_note.ok_or_else(|| anyhow!("Target note not found"))?;

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
        "\x1b[32m✔\x1b[0m Note \x1b[36m{}\x1b[0m marked as \x1b[1m{:?}\x1b[0m",
        &note.id.to_string()[..8],
        note.status
    );

    Ok(())
}
