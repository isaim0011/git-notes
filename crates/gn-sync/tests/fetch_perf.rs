use gn_core::{LwwStrategy, Namespace, Note, NotesEngine};
use gn_sync::fetch::fetch_notes;
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;

#[test]
fn test_fetch_notes_performance() {
    let remote_dir = TempDir::new().unwrap();
    let local_dir = TempDir::new().unwrap();

    let remote_path = remote_dir.path();
    let local_path = local_dir.path();

    // Init remote git repo (bare repo or regular repo)
    Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_path)
        .status()
        .unwrap();

    // Init local git repo
    Command::new("git")
        .args(["init"])
        .current_dir(local_path)
        .status()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(local_path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(local_path)
        .status()
        .unwrap();

    // Add remote to local repo
    Command::new("git")
        .args(["remote", "add", "origin", remote_path.to_str().unwrap()])
        .current_dir(local_path)
        .status()
        .unwrap();

    // Set up notes in remote repo using NotesEngine
    let remote_engine = NotesEngine::new(remote_path);

    let note_count = 100;
    println!("Generating {} notes in remote...", note_count);
    for i in 0..note_count {
        let note = Note::new(
            format!("commit_sha_{}", i),
            Some(format!("file_{}.rs", i)),
            Some(1),
            Some(10),
            format!("Note body content {}", i),
            "Test Author <test@example.com>".to_string(),
            Namespace::Comments,
        );
        remote_engine.write_note(&note).unwrap();
    }

    // Now measure fetch_notes execution time
    println!("Starting fetch_notes benchmark for {} notes...", note_count);
    let start = Instant::now();
    let report = fetch_notes(local_path, "origin", &[Namespace::Comments], &LwwStrategy).unwrap();
    let duration = start.elapsed();

    println!("Report: fetched={}, merged={}", report.fetched, report.merged);
    println!("Time taken to fetch and merge {} notes: {:?}", note_count, duration);

    assert_eq!(report.fetched, note_count);
    assert_eq!(report.merged, note_count);

    // Verify local engine can read all notes
    let local_engine = NotesEngine::new(local_path);
    let local_notes = local_engine.read_notes(&Namespace::Comments).unwrap();
    assert_eq!(local_notes.len(), note_count);
}
