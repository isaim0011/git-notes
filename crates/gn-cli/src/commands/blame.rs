use anyhow::Result;
use clap::Args;
use gn_core::namespace::Namespace;
use gn_core::NotesEngine;
use std::collections::HashMap;
use std::process::Command;

#[derive(Args, Debug)]
pub struct BlameArgs {
    /// File to blame
    #[arg(short, long)]
    pub file: String,

    /// Namespace to check notes from (default: all)
    #[arg(short, long, default_value = "comments")]
    pub namespace: String,
}

pub fn run(args: &BlameArgs) -> Result<()> {
    let engine = NotesEngine::new(".");
    let ns = Namespace::from_str(&args.namespace);

    // Read notes for this namespace
    let notes = engine.read_notes(&ns).unwrap_or_default();

    // Index notes by line start
    let mut notes_by_line: HashMap<u32, Vec<&gn_core::note::Note>> = HashMap::new();
    for note in &notes {
        if let Some(ref note_file) = note.file {
            if note_file == &args.file {
                let line_num = note.line_start.unwrap_or(1);
                notes_by_line.entry(line_num).or_default().push(note);
            }
        }
    }

    // Run git blame
    let blame_output = Command::new("git")
        .args(["blame", &args.file])
        .output()?;

    if !blame_output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&blame_output.stderr));
        return Ok(());
    }

    let blame_text = String::from_utf8_lossy(&blame_output.stdout);
    for (idx, line) in blame_text.lines().enumerate() {
        let current_line_num = (idx + 1) as u32;
        println!("{}", line);

        if let Some(attached_notes) = notes_by_line.get(&current_line_num) {
            for n in attached_notes {
                let status_str = match n.status {
                    gn_core::note::NoteStatus::Open => "\x1b[33m[Open]\x1b[0m",
                    gn_core::note::NoteStatus::Approved => "\x1b[32m[Approved]\x1b[0m",
                    gn_core::note::NoteStatus::Rejected => "\x1b[31m[Rejected]\x1b[0m",
                    gn_core::note::NoteStatus::Resolved => "\x1b[32m[Resolved]\x1b[0m",
                };
                let id_short = &n.id.to_string()[..8];
                println!(
                    "          \x1b[36m╰─ 💬 [{} — {}] {}\x1b[0m {}",
                    n.author, id_short, n.body.trim(), status_str
                );
            }
        }
    }

    Ok(())
}
