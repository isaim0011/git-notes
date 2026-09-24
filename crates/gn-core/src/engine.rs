use crate::error::{GnError, Result};
use crate::namespace::Namespace;
use crate::note::Note;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;
use uuid::Uuid;

pub struct NotesEngine {
    pub repo_path: PathBuf,
}

impl NotesEngine {
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    fn git_cmd(&self) -> Command {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_path);
        cmd
    }

    pub fn read_notes(&self, namespace: &Namespace) -> Result<Vec<Note>> {
        let ref_path = namespace.ref_path();

        let output = self.git_cmd().args(["ls-tree", "-r", &ref_path]).output()?;

        if !output.status.success() {
            // If the ref doesn't exist, we just have 0 notes.
            return Ok(Vec::new());
        }

        let out_str = String::from_utf8(output.stdout)?;
        let mut notes = Vec::new();

        for line in out_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }
            let blob_hash = parts[2];

            let cat_file = self
                .git_cmd()
                .args(["cat-file", "blob", blob_hash])
                .output()?;

            if cat_file.status.success() {
                let blob_str = String::from_utf8(cat_file.stdout)?;
                if let Ok(note) = serde_json::from_str::<Note>(&blob_str) {
                    notes.push(note);
                }
            }
        }

        Ok(notes)
    }

    pub fn write_note(&self, note: &Note) -> Result<String> {
        let commits = self.write_notes(std::slice::from_ref(note))?;
        commits
            .into_iter()
            .next()
            .ok_or_else(|| GnError::Git("Failed to write note".to_string()))
    }

    pub fn write_notes(&self, notes: &[Note]) -> Result<Vec<String>> {
        if notes.is_empty() {
            return Ok(Vec::new());
        }

        // Group notes by namespace while preserving insertion order
        let mut ns_order = Vec::new();
        let mut ns_map: HashMap<Namespace, Vec<&Note>> = HashMap::new();

        for note in notes {
            let entry = ns_map.entry(note.namespace.clone());
            if matches!(entry, std::collections::hash_map::Entry::Vacant(_)) {
                ns_order.push(note.namespace.clone());
            }
            entry.or_default().push(note);
        }

        let mut commit_hashes = Vec::new();

        for ns in ns_order {
            let ns_notes = match ns_map.get(&ns) {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };

            // 1. Write blobs in batch using temp files & git hash-object
            let temp_dir = TempDir::new()?;
            let mut file_paths = Vec::with_capacity(ns_notes.len());

            for (idx, note) in ns_notes.iter().enumerate() {
                let note_json = serde_json::to_string(note)?;
                let file_path = temp_dir.path().join(format!("note_{}.json", idx));
                fs::write(&file_path, note_json.as_bytes())?;
                file_paths.push(file_path);
            }

            // Chunk file paths to avoid OS arg length limits (500 files per invocation)
            let mut blob_hashes = Vec::with_capacity(ns_notes.len());
            for chunk in file_paths.chunks(500) {
                let mut hash_cmd = self.git_cmd();
                hash_cmd.args(["hash-object", "-w"]);
                for p in chunk {
                    hash_cmd.arg(p);
                }

                let hash_output = hash_cmd.output()?;
                if !hash_output.status.success() {
                    return Err(GnError::Git(format!(
                        "git hash-object failed: {}",
                        String::from_utf8_lossy(&hash_output.stderr)
                    )));
                }

                let out_str = String::from_utf8(hash_output.stdout)?;
                for line in out_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        blob_hashes.push(trimmed.to_string());
                    }
                }
            }

            if blob_hashes.len() != ns_notes.len() {
                return Err(GnError::Git(format!(
                    "Mismatch in hash-object output count: expected {}, got {}",
                    ns_notes.len(),
                    blob_hashes.len()
                )));
            }

            let ref_path = ns.ref_path();

            // 2. Read existing tree or create new
            let tree_cmd = self.git_cmd().args(["ls-tree", &ref_path]).output()?;
            let mut tree_entries = String::new();
            if tree_cmd.status.success() {
                tree_entries = String::from_utf8(tree_cmd.stdout)?;
            }

            // Set of note IDs updated/added in this namespace batch
            let updated_ids: HashSet<String> = ns_notes.iter().map(|n| n.id.to_string()).collect();

            // Filter out existing entries for note IDs updated in this batch
            let mut new_tree_input = String::new();
            for line in tree_entries.lines() {
                if let Some(filename) = line.split_whitespace().last() {
                    if updated_ids.contains(filename) {
                        continue;
                    }
                }
                new_tree_input.push_str(line);
                new_tree_input.push('\n');
            }

            // Append new entries
            for (note, blob_hash) in ns_notes.iter().zip(blob_hashes.iter()) {
                new_tree_input.push_str(&format!("100644 blob {}\t{}\n", blob_hash, note.id));
            }

            // 3. mktree
            let mut mktree_cmd = self.git_cmd();
            mktree_cmd
                .arg("mktree")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());

            let mut child = mktree_cmd.spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(new_tree_input.as_bytes())?;
            }
            let mktree_output = child.wait_with_output()?;
            if !mktree_output.status.success() {
                return Err(GnError::Git(format!(
                    "git mktree failed: {}",
                    String::from_utf8_lossy(&mktree_output.stderr)
                )));
            }
            let new_tree_hash = String::from_utf8(mktree_output.stdout)?.trim().to_string();

            // 4. commit-tree
            let mut commit_cmd = self.git_cmd();
            let commit_msg = if ns_notes.len() == 1 {
                format!("Update note {}", ns_notes[0].id)
            } else {
                format!("Update {} notes", ns_notes.len())
            };

            commit_cmd.args(["commit-tree", &new_tree_hash, "-m", &commit_msg]);

            // Find parent commit if ref exists
            let rev_parse = self
                .git_cmd()
                .args(["rev-parse", "-q", "--verify", &ref_path])
                .output()?;
            if rev_parse.status.success() {
                let parent_hash = String::from_utf8(rev_parse.stdout)?.trim().to_string();
                if !parent_hash.is_empty() {
                    commit_cmd.args(["-p", &parent_hash]);
                }
            }

            let commit_output = commit_cmd.output()?;
            if !commit_output.status.success() {
                return Err(GnError::Git(format!(
                    "git commit-tree failed: {}",
                    String::from_utf8_lossy(&commit_output.stderr)
                )));
            }
            let commit_hash = String::from_utf8(commit_output.stdout)?.trim().to_string();

            // 5. update-ref
            let update_ref_output = self
                .git_cmd()
                .args(["update-ref", &ref_path, &commit_hash])
                .output()?;
            if !update_ref_output.status.success() {
                return Err(GnError::Git(format!(
                    "git update-ref failed: {}",
                    String::from_utf8_lossy(&update_ref_output.stderr)
                )));
            }

            commit_hashes.push(commit_hash);
        }

        Ok(commit_hashes)
    }

    pub fn delete_note(&self, note_id: Uuid, namespace: &Namespace) -> Result<()> {
        let ref_path = namespace.ref_path();

        let tree_cmd = self.git_cmd().args(["ls-tree", &ref_path]).output()?;
        if !tree_cmd.status.success() {
            return Err(GnError::NotFound(format!(
                "Namespace {} not found",
                ref_path
            )));
        }

        let tree_entries = String::from_utf8(tree_cmd.stdout)?;
        let mut new_tree_input = String::new();
        let mut found = false;

        for line in tree_entries.lines() {
            if line.ends_with(&note_id.to_string()) {
                found = true;
            } else {
                new_tree_input.push_str(line);
                new_tree_input.push('\n');
            }
        }

        if !found {
            return Err(GnError::NotFound(format!("Note {} not found", note_id)));
        }

        let mut mktree_cmd = self.git_cmd();
        mktree_cmd
            .arg("mktree")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        let mut child = mktree_cmd.spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(new_tree_input.as_bytes())?;
        }
        let mktree_output = child.wait_with_output()?;
        let new_tree_hash = String::from_utf8(mktree_output.stdout)?.trim().to_string();

        let mut commit_cmd = self.git_cmd();
        commit_cmd.args([
            "commit-tree",
            &new_tree_hash,
            "-m",
            &format!("Delete note {}", note_id),
        ]);

        let rev_parse = self.git_cmd().args(["rev-parse", &ref_path]).output()?;
        if rev_parse.status.success() {
            let parent_hash = String::from_utf8(rev_parse.stdout)?.trim().to_string();
            commit_cmd.args(["-p", &parent_hash]);
        }

        let commit_output = commit_cmd.output()?;
        let commit_hash = String::from_utf8(commit_output.stdout)?.trim().to_string();

        self.git_cmd()
            .args(["update-ref", &ref_path, &commit_hash])
            .output()?;

        Ok(())
    }

    pub fn list_notes_for_commit(&self, commit: &str, namespace: &Namespace) -> Result<Vec<Note>> {
        let all_notes = self.read_notes(namespace)?;
        let filtered = all_notes
            .into_iter()
            .filter(|n| n.commit == commit)
            .collect();
        Ok(filtered)
    }
}
