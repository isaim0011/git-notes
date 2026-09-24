use gn_core::{LwwStrategy, Namespace, Note, NotesEngine};
use gn_sync::fetch::fetch_notes;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_fetch_notes() {
    let remote_dir = TempDir::new().unwrap();
    let local_dir = TempDir::new().unwrap();

    // Init remote git repo (bare repo)
    Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_dir.path())
        .status()
        .unwrap();

    // Init local git repo
    Command::new("git")
        .args(["init"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    // Add remote to local
    Command::new("git")
        .args(["remote", "add", "origin", remote_dir.path().to_str().unwrap()])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    // Create a note locally
    let engine = NotesEngine::new(local_dir.path());
    let note = Note::new(
        "commit123".to_string(),
        None,
        None,
        None,
        "Remote Note Content".to_string(),
        "Tester <test@test.com>".to_string(),
        Namespace::Comments,
    );
    engine.write_note(&note).unwrap();

    // Push local note to remote as origin's comments namespace
    Command::new("git")
        .args(["push", "origin", "refs/notes/comments:refs/notes/comments"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    // Clear local comments ref by deleting it
    Command::new("git")
        .args(["update-ref", "-d", "refs/notes/comments"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    assert!(engine.read_notes(&Namespace::Comments).unwrap().is_empty());

    // Now fetch notes from origin
    let strategy = LwwStrategy;
    let report = fetch_notes(
        local_dir.path(),
        "origin",
        &[Namespace::Comments],
        &strategy,
    )
    .unwrap();

    assert_eq!(report.fetched, 1);
    assert_eq!(report.merged, 1);

    let fetched_notes = engine.read_notes(&Namespace::Comments).unwrap();
    assert_eq!(fetched_notes.len(), 1);
    assert_eq!(fetched_notes[0].id, note.id);
    assert_eq!(fetched_notes[0].body, "Remote Note Content");
}
