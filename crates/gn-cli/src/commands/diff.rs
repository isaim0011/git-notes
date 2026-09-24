use anyhow::Result;
use clap::Args;
use gn_core::namespace::Namespace;
use gn_core::NotesEngine;
use std::collections::HashMap;
use std::process::Command;

#[derive(Args, Debug)]
pub struct DiffArgs {
    /// Target commit, branch, or file to diff
    pub target: Option<String>,

    /// Namespace to check notes from (default: all)
    #[arg(short, long, default_value = "comments")]
    pub namespace: String,
}

pub fn run(args: &DiffArgs) -> Result<()> {
    let engine = NotesEngine::new(".");
    let ns = Namespace::from_str(&args.namespace);
    let notes = engine.read_notes(&ns).unwrap_or_default();

    // Map: file -> list of notes
    let mut notes_by_file: HashMap<String, Vec<&gn_core::note::Note>> = HashMap::new();
    for note in &notes {
        if let Some(ref f) = note.file {
            notes_by_file.entry(f.clone()).or_default().push(note);
        }
    }

    let mut git_cmd = Command::new("git");
    git_cmd.arg("diff");
    if let Some(ref t) = args.target {
        git_cmd.arg(t);
    }

    let output = git_cmd.output()?;
    let diff_text = String::from_utf8_lossy(&output.stdout);

    if diff_text.trim().is_empty() {
        println!("No diff found.");
        return Ok(());
    }

    let mut current_file: Option<String> = None;

    for line in diff_text.lines() {
        if line.starts_with("+++ b/") {
            let f = line.trim_start_matches("+++ b/").to_string();
            current_file = Some(f);
            println!("{}", line);
            continue;
        }

        if line.starts_with("@@ ") {
            println!("\x1b[36m{}\x1b[0m", line);
            // Check if current file has notes
            if let Some(ref f) = current_file {
                if let Some(file_notes) = notes_by_file.get(f) {
                    for n in file_notes {
                        let id_short = &n.id.to_string()[..8];
                        let line_str = n.line_start.map(|l| format!(":{}", l)).unwrap_or_default();
                        println!(
                            "  \x1b[33m💬 Note [{}] on {}{} by {}: \"{}\"\x1b[0m",
                            id_short,
                            f,
                            line_str,
                            n.author.split('<').next().unwrap_or(&n.author).trim(),
                            n.body.replace('\n', " ").chars().take(60).collect::<String>()
                        );
                    }
                }
            }
            continue;
        }

        if line.starts_with('+') {
            println!("\x1b[32m{}\x1b[0m", line);
        } else if line.starts_with('-') {
            println!("\x1b[31m{}\x1b[0m", line);
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}
