use anyhow::Result;
use chrono::{DateTime, Utc};
use gn_core::{MergeStrategy, Namespace, NotesEngine};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    pub fetched: usize,
    pub merged: usize,
    pub conflicts: usize,
    pub timestamp: DateTime<Utc>,
}

pub fn fetch_notes(
    repo_path: &Path,
    remote: &str,
    namespaces: &[Namespace],
    strategy: &dyn MergeStrategy,
) -> Result<SyncReport> {
    let engine = NotesEngine::new(repo_path);
    let mut total_fetched = 0;
    let mut total_merged = 0;

    for ns in namespaces {
        let ref_path = ns.ref_path();

        // Fetch into a temporary remote ref
        let remote_ref = format!("refs/notes/{}_{}_remote", remote, ns);
        let refspec = format!("{}:{}", ref_path, remote_ref);

        let _ = Command::new("git")
            .current_dir(repo_path)
            .args(["fetch", remote, &refspec])
            .status(); // ignore failure if remote ref doesn't exist

        let local_notes = engine.read_notes(ns).unwrap_or_default();
        let remote_notes = engine.read_notes_ref(&remote_ref).unwrap_or_default();

        total_fetched += remote_notes.len();

        if !remote_notes.is_empty() {
            let merged_notes = strategy.merge(&local_notes, &remote_notes);

            // Re-write merged notes locally
            // First we need to delete existing local ref or overwrite
            // An easy approach is writing all notes. The LWW strategy guarantees identical notes have same hash
            for note in &merged_notes {
                engine.write_note(note)?;
            }
            total_merged += merged_notes.len();
        }

        // Clean up remote ref
        let _ = Command::new("git")
            .current_dir(repo_path)
            .args(["update-ref", "-d", &remote_ref])
            .status();
    }

    Ok(SyncReport {
        fetched: total_fetched,
        merged: total_merged,
        conflicts: 0, // simple LWW handles conflicts transparently for now
        timestamp: Utc::now(),
    })
}
