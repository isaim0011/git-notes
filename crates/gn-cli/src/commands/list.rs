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
        // match on string representation
        all_notes.retain(|n| format!("{:?}", n.status).to_lowercase() == status.to_lowercase());
    }
    if let Some(author) = &args.author {
        all_notes.retain(|n| n.author.contains(author));
    }

    if args.json {
        let json = serde_json::to_string_pretty(&all_notes)?;
        println!("{}", json);
    } else {
        println!(
            "{:<10} | {:<20} | {:<20} | {:<25} | {:<10} | {}",
            "ID", "FILE:LINE", "AUTHOR", "DATE", "STATUS", "BODY"
        );
        for note in all_notes {
            let id_short = note.id.to_string().chars().take(8).collect::<String>();
            let file = note.file.clone().unwrap_or_else(|| "".to_string());
            let line = note.line_start.unwrap_or(0);
            let file_line = format!("{}:{}", file, line);
            let body_short = if note.body.len() > 60 {
                format!("{}...", &note.body[..57])
            } else {
                note.body.clone()
            };
            println!(
                "{:<10} | {:<20} | {:<20} | {:<25} | {:<10?} | {}",
                id_short, file_line, note.author, note.timestamp, note.status, body_short
            );
        }
    }

    Ok(())
}
