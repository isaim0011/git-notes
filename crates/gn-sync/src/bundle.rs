use anyhow::{Context, Result};
use gn_core::{MergeStrategy, Namespace, NotesEngine};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleReport {
    pub path: PathBuf,
    pub refs_count: usize,
    pub fetched: usize,
    pub merged: usize,
}

/// Create a portable git bundle containing all `refs/notes/*` for transfer over USB or AirDrop.
pub fn export_bundle(repo_path: &Path, output: &Path) -> Result<BundleReport> {
    // 1. Discover all refs under refs/notes/
    let refs_output = Command::new("git")
        .current_dir(repo_path)
        .args(["for-each-ref", "--format=%(refname)", "refs/notes/"])
        .output()
        .context("Failed to query notes refs")?;

    if !refs_output.status.success() {
        let err = String::from_utf8_lossy(&refs_output.stderr);
        anyhow::bail!("Failed to inspect notes refs: {}", err.trim());
    }

    let refs_str = String::from_utf8_lossy(&refs_output.stdout);
    let note_refs: Vec<String> = refs_str
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| {
            l.starts_with("refs/notes/")
                && !l.contains("/bundle_")
                && !l.contains("/imported_")
                && !l.contains("/p2p_")
                && !l.ends_with("_remote")
        })
        .collect();

    if note_refs.is_empty() {
        anyhow::bail!("No notes refs found in repository to export. Create notes first using 'gn add'.");
    }

    // 2. Resolve output path
    let resolved_output = if output.is_relative() {
        std::env::current_dir()
            .unwrap_or_else(|_| repo_path.to_path_buf())
            .join(output)
    } else {
        output.to_path_buf()
    };

    if let Some(parent) = resolved_output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for bundle: {}", parent.display()))?;
    }

    // 3. Create bundle: git bundle create <output> <ref1> <ref2> ...
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_path);
    cmd.arg("bundle").arg("create").arg(&resolved_output);
    for r in &note_refs {
        cmd.arg(r);
    }

    let create_output = cmd.output().context("Failed to execute git bundle create")?;
    if !create_output.status.success() {
        let err = String::from_utf8_lossy(&create_output.stderr);
        anyhow::bail!("Failed to create git bundle: {}", err.trim());
    }

    Ok(BundleReport {
        path: resolved_output,
        refs_count: note_refs.len(),
        fetched: 0,
        merged: 0,
    })
}

/// Fetch and merge notes from a portable `.bundle` file.
pub fn import_bundle(
    repo_path: &Path,
    input: &Path,
    strategy: &dyn MergeStrategy,
) -> Result<BundleReport> {
    let resolved_input = if input.is_relative() {
        std::env::current_dir()
            .unwrap_or_else(|_| repo_path.to_path_buf())
            .join(input)
    } else {
        input.to_path_buf()
    };

    if !resolved_input.exists() {
        anyhow::bail!("Bundle file not found: {}", resolved_input.display());
    }

    // 1. Verify bundle
    let verify = Command::new("git")
        .current_dir(repo_path)
        .args(["bundle", "verify"])
        .arg(&resolved_input)
        .output()
        .context("Failed to verify git bundle")?;

    if !verify.status.success() {
        let err = String::from_utf8_lossy(&verify.stderr);
        anyhow::bail!(
            "Invalid git bundle at {}: {}",
            resolved_input.display(),
            err.trim()
        );
    }

    // 2. Fetch refs from bundle into an isolated temporary namespace
    let temp_token = format!("bundle_{}", Uuid::new_v4().simple());
    let refspec = format!("refs/notes/*:refs/notes/{}/*", temp_token);

    let fetch_out = Command::new("git")
        .current_dir(repo_path)
        .args(["fetch"])
        .arg(&resolved_input)
        .arg(&refspec)
        .output()
        .context("Failed to fetch from bundle")?;

    if !fetch_out.status.success() {
        let err = String::from_utf8_lossy(&fetch_out.stderr);
        anyhow::bail!("Failed to fetch notes from bundle: {}", err.trim());
    }

    // 3. Discover all imported temporary refs
    let temp_ref_prefix = format!("refs/notes/{}/", temp_token);
    let list_out = Command::new("git")
        .current_dir(repo_path)
        .args(["for-each-ref", "--format=%(refname)", &temp_ref_prefix])
        .output()
        .context("Failed to list imported refs")?;

    let imported_refs: Vec<String> = String::from_utf8_lossy(&list_out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| l.starts_with(&temp_ref_prefix))
        .collect();

    let engine = NotesEngine::new(repo_path);
    let mut total_fetched = 0;
    let mut total_merged = 0;

    for temp_ref in &imported_refs {
        let ns_suffix = temp_ref
            .strip_prefix(&temp_ref_prefix)
            .unwrap_or(temp_ref);

        let target_ns = Namespace::from_str(ns_suffix);
        let temp_ns = Namespace::Custom(format!("{}/{}", temp_token, ns_suffix));

        let local_notes = engine.read_notes(&target_ns).unwrap_or_default();
        let imported_notes = engine.read_notes(&temp_ns).unwrap_or_default();

        total_fetched += imported_notes.len();

        if !imported_notes.is_empty() {
            let merged = strategy.merge(&local_notes, &imported_notes);
            for note in &merged {
                let mut note_to_write = note.clone();
                note_to_write.namespace = target_ns.clone();
                engine.write_note(&note_to_write)?;
            }
            total_merged += merged.len();
        }

        // Clean up temporary ref
        let _ = Command::new("git")
            .current_dir(repo_path)
            .args(["update-ref", "-d", temp_ref])
            .status();
    }

    Ok(BundleReport {
        path: resolved_input,
        refs_count: imported_refs.len(),
        fetched: total_fetched,
        merged: total_merged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gn_core::{LwwStrategy, Note};
    use std::fs;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("gn_bundle_test_{}", Uuid::new_v4().simple()));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn run_git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(dir)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    #[test]
    fn test_export_and_import_bundle() {
        let repo_a = TestDir::new();
        let repo_b = TestDir::new();
        let bundle_dir = TestDir::new();
        let bundle_file = bundle_dir.path().join("notes.bundle");

        // Init repo A
        run_git(repo_a.path(), &["init"]);
        run_git(repo_a.path(), &["config", "user.name", "Test User"]);
        run_git(repo_a.path(), &["config", "user.email", "test@example.com"]);
        fs::write(repo_a.path().join("file.txt"), "hello world\n").unwrap();
        run_git(repo_a.path(), &["add", "."]);
        run_git(repo_a.path(), &["commit", "-m", "Initial commit"]);

        // Init repo B with same commit
        run_git(repo_b.path(), &["init"]);
        run_git(repo_b.path(), &["config", "user.name", "Peer User"]);
        run_git(repo_b.path(), &["config", "user.email", "peer@example.com"]);
        fs::write(repo_b.path().join("file.txt"), "hello world\n").unwrap();
        run_git(repo_b.path(), &["add", "."]);
        run_git(repo_b.path(), &["commit", "-m", "Initial commit"]);

        let engine_a = NotesEngine::new(repo_a.path());
        let note = Note::new(
            "HEAD".to_string(),
            Some("file.txt".to_string()),
            Some(1),
            Some(1),
            "Important note for bundle export".to_string(),
            "Author <a@b.com>".to_string(),
            Namespace::Comments,
        );
        engine_a.write_note(&note).unwrap();

        // Export bundle from repo A
        let export_report = export_bundle(repo_a.path(), &bundle_file).unwrap();
        assert_eq!(export_report.refs_count, 1);
        assert!(bundle_file.exists());

        // Import bundle into repo B
        let strategy = LwwStrategy;
        let import_report = import_bundle(repo_b.path(), &bundle_file, &strategy).unwrap();
        assert_eq!(import_report.refs_count, 1);
        assert_eq!(import_report.fetched, 1);
        assert_eq!(import_report.merged, 1);

        // Verify note is in repo B
        let engine_b = NotesEngine::new(repo_b.path());
        let notes_b = engine_b.read_notes(&Namespace::Comments).unwrap();
        assert_eq!(notes_b.len(), 1);
        assert_eq!(notes_b[0].body, "Important note for bundle export");
    }
}
