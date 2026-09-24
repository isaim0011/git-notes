use anyhow::Result;
use clap::Args;
use gn_core::NotesEngine;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;

#[derive(Args)]
pub struct ExportArgs {
    /// Format (json, html, markdown)
    #[arg(short, long, default_value = "html")]
    pub format: String,

    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub out_dir: String,
}

pub fn generate_markdown(all_notes: &[gn_core::Note]) -> String {
    let mut md = String::from("# Git Notes\n\n");

    let mut notes_by_file: BTreeMap<&str, Vec<&gn_core::Note>> = BTreeMap::new();

    for note in all_notes {
        if let Some(file) = note.file.as_deref() {
            notes_by_file.entry(file).or_default().push(note);
        }
    }

    for (file, notes) in notes_by_file {
        let _ = write!(md, "## {}\n\n", file);
        for note in notes {
            let _ = write!(
                md,
                "### Line {} - {} ({})\n\n",
                note.line_start.unwrap_or(0),
                note.author,
                note.timestamp
            );
            let _ = write!(md, "**Status:** {:?}\n\n", note.status);
            let _ = write!(md, "{}\n\n", note.body);
        }
    }

    md
}

pub fn run(args: &ExportArgs) -> Result<()> {
    let engine = NotesEngine::new(".");
    let namespaces = vec!["comments", "review", "todos"];

    let mut all_notes = Vec::new();
    for ns in &namespaces {
        let namespace_enum = gn_core::Namespace::Custom(ns.to_string());
        if let Ok(notes) = engine.read_notes(&namespace_enum) {
            all_notes.extend(notes);
        }
    }

    let out_path = std::path::Path::new(&args.out_dir);
    if !out_path.exists() {
        fs::create_dir_all(out_path)?;
    }

    match args.format.to_lowercase().as_str() {
        "json" => {
            let file_path = out_path.join("notes.json");
            let json = serde_json::to_string_pretty(&all_notes)?;
            fs::write(&file_path, json)?;
            println!("✓ Exported notes to {}", file_path.display());
        }
        "markdown" => {
            let file_path = out_path.join("NOTES.md");
            let md = generate_markdown(&all_notes);
            fs::write(&file_path, md)?;
            println!("✓ Exported notes to {}", file_path.display());
        }
        "html" => {
            let file_path = out_path.join("index.html");
            let json = serde_json::to_string(&all_notes)?;
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Git Notes</title>
    <style>
        body {{ font-family: sans-serif; margin: 0; padding: 20px; }}
        .note {{ border: 1px solid #ccc; padding: 10px; margin-bottom: 10px; border-radius: 4px; }}
        .header {{ color: #555; font-size: 0.9em; margin-bottom: 5px; }}
        .body {{ white-space: pre-wrap; }}
    </style>
</head>
<body>
    <h1>Git Notes</h1>
    <div id="notes"></div>
    <script>
        const notes = {};
        const container = document.getElementById('notes');
        notes.forEach(note => {{
            const file = note.file || "";
            const line = note.line_start || 0;
            const div = document.createElement('div');
            div.className = 'note';
            div.innerHTML = `
                <div class="header"><strong>${{file}}:${{line}}</strong> by ${{note.author}} on ${{note.timestamp}} [${{note.status}}]</div>
                <div class="body">${{note.body}}</div>
            `;
            container.appendChild(div);
        }});
    </script>
</body>
</html>"#,
                json
            );
            fs::write(&file_path, html)?;
            println!("✓ Exported notes to {}", file_path.display());
        }
        _ => {
            anyhow::bail!("Unsupported format: {}", args.format);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gn_core::{Namespace, Note};
    use std::time::Instant;

    fn make_sample_note(file: &str, line: u32, body: &str) -> Note {
        Note::new(
            "commit123".to_string(),
            Some(file.to_string()),
            Some(line),
            None,
            body.to_string(),
            "Author <author@example.com>".to_string(),
            Namespace::Custom("comments".to_string()),
        )
    }

    pub fn generate_markdown_unoptimized(all_notes: &[Note]) -> String {
        let mut md = String::from("# Git Notes\n\n");

        // Group by file
        let mut files: Vec<String> = all_notes.iter().filter_map(|n| n.file.clone()).collect();
        files.sort();
        files.dedup();

        for file in files {
            md.push_str(&format!("## {}\n\n", file));
            for note in all_notes
                .iter()
                .filter(|n| n.file.as_deref() == Some(file.as_str()))
            {
                md.push_str(&format!(
                    "### Line {} - {} ({})\n\n",
                    note.line_start.unwrap_or(0),
                    note.author,
                    note.timestamp
                ));
                md.push_str(&format!("**Status:** {:?}\n\n", note.status));
                md.push_str(&format!("{}\n\n", note.body));
            }
        }
        md
    }

    #[test]
    fn test_export_markdown_correctness() {
        let notes = vec![
            make_sample_note("src/b.rs", 10, "Note B"),
            make_sample_note("src/a.rs", 5, "Note A"),
            make_sample_note("src/b.rs", 20, "Note B2"),
        ];

        let md = generate_markdown(&notes);
        assert!(md.contains("## src/a.rs"));
        assert!(md.contains("## src/b.rs"));
        assert!(md.contains("Note A"));
        assert!(md.contains("Note B"));
        assert!(md.contains("Note B2"));
    }

    #[test]
    fn test_benchmark_markdown_export() {
        // Generate 5,000 notes spread across 500 files
        let mut notes = Vec::with_capacity(5_000);
        for f in 0..500 {
            let filename = format!("src/file_{:04}.rs", f);
            for i in 0..10 {
                notes.push(make_sample_note(&filename, i * 10, &format!("Note {}", i)));
            }
        }

        let start = Instant::now();
        let unopt_out = generate_markdown_unoptimized(&notes);
        let unopt_dur = start.elapsed();

        let start = Instant::now();
        let opt_out = generate_markdown(&notes);
        let opt_dur = start.elapsed();

        println!("\n================ PERFORMANCE COMPARISON ================");
        println!("Unoptimized generation time (5,000 notes, 500 files): {:?}", unopt_dur);
        println!("Optimized generation time (5,000 notes, 500 files):   {:?}", opt_dur);
        println!(
            "Speedup factor:                                       {:.2}x",
            unopt_dur.as_secs_f64() / opt_dur.as_secs_f64()
        );
        println!("======================================================\n");

        assert_eq!(unopt_out, opt_out);
    }
}
