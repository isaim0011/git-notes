use anyhow::Result;
use clap::Args;
use gn_core::NotesEngine;

#[derive(Args)]
pub struct ListArgs {
    /// Filter by file
    #[arg(short, long)]
    pub file: Option<String>,

    /// Filter by commit
    #[arg(short, long)]
    pub commit: Option<String>,

    /// Filter by status
    #[arg(short, long)]
    pub status: Option<String>,

    /// Filter by author
    #[arg(short, long)]
    pub author: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Namespace to list from
    #[arg(short, long)]
    pub namespace: Option<String>,
}

pub fn run(args: &ListArgs) -> Result<()> {
    let engine = NotesEngine::new(".");
    let namespaces = match &args.namespace {
        Some(ns) => vec![ns.clone()],
        None => vec![
            "comments".to_string(),
            "review".to_string(),
            "todos".to_string(),
        ],
    };

    let mut all_notes = Vec::new();

    for ns in namespaces {
        let namespace_enum = gn_core::Namespace::Custom(ns.clone());
        if let Ok(mut notes) = engine.read_notes(&namespace_enum) {
            all_notes.append(&mut notes);
        }
    }

    if let Some(file) = &args.file {
        all_notes.retain(|n| n.file.as_deref() == Some(file));
    }
    if let Some(commit) = &args.commit {
        all_notes.retain(|n| n.commit == *commit);
    }
    if let Some(status) = &args.status {
        all_notes.retain(|n| format!("{:?}", n.status).to_lowercase() == status.to_lowercase());
    }
    if let Some(author) = &args.author {
        all_notes.retain(|n| n.author.contains(author));
    }

    if args.json {
        let json = serde_json::to_string_pretty(&all_notes)?;
        println!("{}", json);
    } else {
        if all_notes.is_empty() {
            println!("No notes found.");
            return Ok(());
        }

        println!(
            "{:<5} {:<10} {:<22} {:<18} {:<10} {}",
            "#", "ID", "FILE:LINE", "AUTHOR", "STATUS", "BODY"
        );
        println!("{}", "─".repeat(88));

        for (idx, note) in all_notes.iter().enumerate() {
            let num = format!("[{}]", idx + 1);
            let id_short = note.id.to_string().chars().take(8).collect::<String>();
            let file = note.file.clone().unwrap_or_else(|| "".to_string());
            let line = note.line_start.unwrap_or(0);
            let file_line = if file.is_empty() {
                "-".to_string()
            } else {
                format!("{}:{}", file, line)
            };
            let body_short = if note.body.len() > 50 {
                format!("{}...", &note.body[..47].replace('\n', " "))
            } else {
                note.body.replace('\n', " ")
            };
            let author_short = note.author.split('<').next().unwrap_or(&note.author).trim();
            let status_str = match note.status {
                gn_core::note::NoteStatus::Open => "\x1b[33mOpen\x1b[0m",
                gn_core::note::NoteStatus::Approved => "\x1b[32mApproved\x1b[0m",
                gn_core::note::NoteStatus::Rejected => "\x1b[31mRejected\x1b[0m",
                gn_core::note::NoteStatus::Resolved => "\x1b[32mResolved\x1b[0m",
            };

            println!(
                "\x1b[1;36m{:<5}\x1b[0m {:<10} {:<22} {:<18} {:<19} {}",
                num, id_short, file_line, author_short, status_str, body_short
            );
        }
        println!("\n\x1b[90mTip: Reply or resolve using numbers: gn r 1 -m \"...\" or gn ok 1\x1b[0m");
    }

    Ok(())
}
