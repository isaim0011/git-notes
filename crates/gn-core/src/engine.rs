use crate::error::{GnError, Result};
use crate::namespace::Namespace;
use crate::note::Note;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
        let mut blob_hashes = Vec::new();

        for line in out_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                blob_hashes.push(parts[2].to_string());
            }
        }

        if blob_hashes.is_empty() {
            return Ok(Vec::new());
        }

        let mut cat_file_cmd = self.git_cmd();
        cat_file_cmd
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut child = cat_file_cmd.spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            std::thread::spawn(move || {
                for hash in blob_hashes {
                    if writeln!(stdin, "{}", hash).is_err() {
                        break;
                    }
                }
            });
        }

        let cat_file_output = child.wait_with_output()?;
        if !cat_file_output.status.success() {
            return Ok(Vec::new());
        }

        let mut notes = Vec::new();
        let stdout = cat_file_output.stdout;
        let mut cursor = 0;

        while cursor < stdout.len() {
            // Find end of header line
            let relative_newline = match stdout[cursor..].iter().position(|&b| b == b'\n') {
                Some(pos) => pos,
                None => break,
            };

            let header_line = match std::str::from_utf8(&stdout[cursor..cursor + relative_newline])
            {
                Ok(s) => s.trim(),
                Err(_) => break,
            };

            cursor += relative_newline + 1;

            let parts: Vec<&str> = header_line.split_whitespace().collect();
            if parts.len() < 3 {
                // If it's missing (e.g., "<hash> missing"), no payload bytes follow
                continue;
            }

            let size: usize = match parts[2].parse() {
                Ok(s) => s,
                Err(_) => break,
            };

            if cursor + size > stdout.len() {
                break;
            }

            let blob_data = &stdout[cursor..cursor + size];
            cursor += size;

            // Consume trailing newline after payload if present
            if cursor < stdout.len() && stdout[cursor] == b'\n' {
                cursor += 1;
            }

            if let Ok(blob_str) = std::str::from_utf8(blob_data) {
                if let Ok(note) = serde_json::from_str::<Note>(blob_str) {
                    notes.push(note);
                }
            }
        }

        Ok(notes)
    }

    pub fn write_note(&self, note: &Note) -> Result<String> {
        let note_json = serde_json::to_string(note)?;

        // 1. Hash object
        let mut hash_cmd = self.git_cmd();
        hash_cmd
            .args(["hash-object", "-w", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut child = hash_cmd.spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(note_json.as_bytes())?;
        }
        let hash_output = child.wait_with_output()?;
        let blob_hash = String::from_utf8(hash_output.stdout)?.trim().to_string();

        let ref_path = note.namespace.ref_path();

        // 2. Read existing tree or create new
        let tree_cmd = self.git_cmd().args(["ls-tree", &ref_path]).output()?;

        let mut tree_entries = String::new();
        if tree_cmd.status.success() {
            tree_entries = String::from_utf8(tree_cmd.stdout)?;
        }

        // Add or update the file named by UUID
        let new_entry = format!("100644 blob {}\t{}\n", blob_hash, note.id);

        // Filter out existing entry for this note id if updating
        let mut new_tree_input = tree_entries
            .lines()
            .filter(|line| !line.ends_with(&note.id.to_string()))
            .map(|line| format!("{}\n", line))
            .collect::<String>();

        new_tree_input.push_str(&new_entry);

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
        let new_tree_hash = String::from_utf8(mktree_output.stdout)?.trim().to_string();

        // 4. commit-tree
        let mut commit_cmd = self.git_cmd();
        commit_cmd.args([
            "commit-tree",
            &new_tree_hash,
            "-m",
            &format!("Update note {}", note.id),
        ]);

        // Find parent commit if ref exists
        let rev_parse = self
            .git_cmd()
            .args(["rev-parse", "-q", "--verify", &ref_path])
            .output()?;
        if rev_parse.status.success() {
            let parent_hash = String::from_utf8(rev_parse.stdout)?.trim().to_string();
            commit_cmd.args(["-p", &parent_hash]);
        }

        let commit_output = commit_cmd.output()?;
        let commit_hash = String::from_utf8(commit_output.stdout)?.trim().to_string();

        // 5. update-ref
        self.git_cmd()
            .args(["update-ref", &ref_path, &commit_hash])
            .output()?;

        Ok(commit_hash)
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
