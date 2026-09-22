use anyhow::Result;
use clap::Args;
use gn_core::NotesEngine;
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
