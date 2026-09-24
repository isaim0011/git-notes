use gn_core::{Namespace, Note, NotesEngine};
use std::process::Command;
use tempfile::TempDir;

fn setup_repo() -> (TempDir, NotesEngine) {
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
    (temp_dir, engine)
}

#[test]
fn test_engine_write_and_read() {
    let (_temp_dir, engine) = setup_repo();

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
fn test_engine_batch_write_and_read() {
    let (_temp_dir, engine) = setup_repo();

    let note1 = Note::new(
        "sha1".to_string(),
        None,
        None,
        None,
        "Note 1".to_string(),
        "Author 1".to_string(),
        Namespace::Comments,
    );
    let note2 = Note::new(
        "sha2".to_string(),
        None,
        None,
        None,
        "Note 2".to_string(),
        "Author 2".to_string(),
        Namespace::Comments,
    );
    let note3 = Note::new(
        "sha3".to_string(),
        None,
        None,
        None,
        "Review Note".to_string(),
        "Author 3".to_string(),
        Namespace::Review,
    );

    let commits = engine.write_notes(&[note1.clone(), note2.clone(), note3.clone()]).unwrap();
    assert_eq!(commits.len(), 2); // 1 for Comments, 1 for Review

    let comments = engine.read_notes(&Namespace::Comments).unwrap();
    assert_eq!(comments.len(), 2);

    let reviews = engine.read_notes(&Namespace::Review).unwrap();
    assert_eq!(reviews.len(), 1);
    assert_eq!(reviews[0].id, note3.id);
}

#[test]
fn test_engine_batch_write_empty() {
    let (_temp_dir, engine) = setup_repo();
    let commits = engine.write_notes(&[]).unwrap();
    assert!(commits.is_empty());
}
