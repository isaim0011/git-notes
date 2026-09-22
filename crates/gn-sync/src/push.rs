use anyhow::{Context, Result};
use gn_core::Namespace;
use std::path::Path;
use std::process::Command;

pub fn push_notes(repo_path: &Path, remote: &str, namespaces: &[Namespace]) -> Result<()> {
    for ns in namespaces {
        let ref_path = ns.ref_path();
        let refspec = format!("{}:{}", ref_path, ref_path);

        let status = Command::new("git")
            .current_dir(repo_path)
            .args(["push", remote, &refspec])
            .status()
            .with_context(|| format!("Failed to push namespace {}", ns))?;

        if !status.success() {
            // It might fail if there's nothing to push or refs don't exist, which could be normal.
            // We can return an error or log a warning.
            tracing::warn!("Push failed for namespace {}", ns);
        }
    }

    Ok(())
}
