use gn_core::{Namespace, Note};
use gn_tui::app::{App, FileEntry};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let mut app = App::new(PathBuf::from(".")).unwrap();

    // Create 100 files and 100,000 notes
    let num_files = 100;
    let notes_per_file = 1000;

    app.file_tree = (0..num_files)
        .map(|i| FileEntry {
            path: format!("src/file_{}.rs", i),
            note_count: notes_per_file,
        })
        .collect();

    let mut notes = Vec::with_capacity(num_files * notes_per_file);
    for i in 0..num_files {
        let file_path = format!("src/file_{}.rs", i);
        for _ in 0..notes_per_file {
            let note = Note::new(
                "commit123".to_string(),
                Some(file_path.clone()),
                Some(10),
                Some(15),
                "Test comment".to_string(),
                "Alice <alice@example.com>".to_string(),
                Namespace::Custom("comments".to_string()),
            );
            notes.push(note);
        }
    }

    app.set_notes(notes);
    app.selected_file = Some(50); // Pick a file in the middle

    // Measure calling current_file_notes 1,000 times (simulating 1000 draw frames)
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        let res = app.current_file_notes();
        std::hint::black_box(res);
    }
    let duration = start.elapsed();

    println!(
        "Total time for {} calls: {:?}, avg per call: {:?}",
        iterations,
        duration,
        duration / iterations
    );
}
