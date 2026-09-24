use gn_core::{Namespace, Note, NotesEngine};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_engine_write_and_read() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Init git repo
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .status()
        .unwrap();

    // Config git to allow commits
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo_path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .status()
        .unwrap();

    let engine = NotesEngine::new(repo_path);
    let note = Note::new(
        "dummy_commit_sha".to_string(),
        None,
        None,
        None,
        "Hello from test".to_string(),
        "Tester <test@test.com>".to_string(),
        Namespace::Comments,
    );

    engine.write_note(&note).unwrap();

    let notes = engine.read_notes(&Namespace::Comments).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, note.id);
    assert_eq!(notes[0].body, "Hello from test");
}

#[test]
fn test_engine_batch_write() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .status()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo_path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .status()
        .unwrap();

    let engine = NotesEngine::new(repo_path);
    let mut batch = Vec::new();

    for i in 0..5 {
        let note = Note::new(
            "dummy_sha".to_string(),
            Some("file.rs".to_string()),
            Some(i),
            Some(i),
            format!("Batch note {}", i),
            "Tester <test@test.com>".to_string(),
            Namespace::Review,
        );
        batch.push(note);
    }

    engine.write_notes(&batch).unwrap();

    let notes = engine.read_notes(&Namespace::Review).unwrap();
    assert_eq!(notes.len(), 5);
}
