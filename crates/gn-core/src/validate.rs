use crate::error::Result;
use crate::namespace::Namespace;
use crate::note::Note;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefValidation {
    pub namespace: String,
    pub ref_path: String,
    pub exists: bool,
    pub commit_sha: Option<String>,
    pub note_count: usize,
    pub corrupt_blobs: usize,
    pub orphan_notes: usize,
    pub unparseable_notes: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationReport {
    pub repo_root: String,
    pub total_refs: usize,
    pub total_notes: usize,
    pub total_corrupt: usize,
    pub total_orphans: usize,
    pub healthy: bool,
    pub ref_reports: Vec<RefValidation>,
}

pub struct DataValidator {
    pub repo_path: PathBuf,
}

impl DataValidator {
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

    /// Check if a commit SHA exists in the Git DAG
    pub fn commit_exists(&self, sha: &str) -> bool {
        if sha.trim().is_empty() {
            return false;
        }
        let status = self.git_cmd()
            .args(["cat-file", "-e", &format!("{}^{{commit}}", sha.trim())])
            .output();
        match status {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    /// Run full integrity audit across all refs/notes/*
    pub fn validate_all(&self, namespaces: &[Namespace]) -> Result<ValidationReport> {
        let mut ref_reports = Vec::new();
        let mut total_notes = 0;
        let mut total_corrupt = 0;
        let mut total_orphans = 0;

        for ns in namespaces {
            let ref_path = ns.ref_path();
            let mut report = RefValidation {
                namespace: ns.to_string(),
                ref_path: ref_path.clone(),
                exists: false,
                commit_sha: None,
                note_count: 0,
                corrupt_blobs: 0,
                orphan_notes: 0,
                unparseable_notes: 0,
                issues: Vec::new(),
            };

            // 1. Check if ref exists and get commit sha
            let rev_parse = self.git_cmd().args(["rev-parse", "--verify", &ref_path]).output()?;
            if !rev_parse.status.success() {
                // Ref does not exist yet (clean state or not initialized)
                ref_reports.push(report);
                continue;
            }

            report.exists = true;
            let ref_commit = String::from_utf8_lossy(&rev_parse.stdout).trim().to_string();
            report.commit_sha = Some(ref_commit);

            // 2. Read git tree blobs
            let ls_tree = self.git_cmd().args(["ls-tree", "-r", &ref_path]).output()?;
            if !ls_tree.status.success() {
                report.issues.push(format!("Corrupt git tree for ref: {}", ref_path));
                report.corrupt_blobs += 1;
                ref_reports.push(report);
                continue;
            }

            let tree_out = String::from_utf8_lossy(&ls_tree.stdout);
            let mut blob_entries = Vec::new(); // (blob_sha, path)
            for line in tree_out.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    blob_entries.push((parts[2].to_string(), parts[3].to_string()));
                }
            }

            report.note_count = blob_entries.len();
            total_notes += blob_entries.len();

            // 3. Inspect each blob: content-addressed integrity & JSON schema
            for (blob_sha, entry_path) in blob_entries {
                let cat_file = self.git_cmd().args(["cat-file", "-p", &blob_sha]).output()?;
                if !cat_file.status.success() {
                    report.corrupt_blobs += 1;
                    report.issues.push(format!("Blob {} at '{}' is unreadable / corrupt", blob_sha, entry_path));
                    continue;
                }

                let content = cat_file.stdout;
                match serde_json::from_slice::<Note>(&content) {
                    Ok(note) => {
                        // Check commit anchor existence
                        if !self.commit_exists(&note.commit) {
                            report.orphan_notes += 1;
                            report.issues.push(format!(
                                "Note {} anchors to unreachable/missing commit {}",
                                note.id,
                                note.commit
                            ));
                        }
                    }
                    Err(err) => {
                        report.unparseable_notes += 1;
                        report.issues.push(format!(
                            "Invalid note JSON schema at blob {} ('{}'): {}",
                            blob_sha, entry_path, err
                        ));
                    }
                }
            }

            total_corrupt += report.corrupt_blobs + report.unparseable_notes;
            total_orphans += report.orphan_notes;
            ref_reports.push(report);
        }

        let repo_root_out = self.git_cmd().args(["rev-parse", "--show-toplevel"]).output()?;
        let repo_root = String::from_utf8_lossy(&repo_root_out.stdout).trim().to_string();

        let healthy = total_corrupt == 0;

        Ok(ValidationReport {
            repo_root,
            total_refs: namespaces.len(),
            total_notes,
            total_corrupt,
            total_orphans,
            healthy,
            ref_reports,
        })
    }
}
