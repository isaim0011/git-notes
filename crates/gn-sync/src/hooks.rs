use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn install_hooks(repo_path: &Path) -> Result<()> {
    let hooks_dir = repo_path.join(".git").join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)?;
    }

    let post_merge_path = hooks_dir.join("post-merge");
    let pre_push_path = hooks_dir.join("pre-push");

    let post_merge_script = r#"#!/bin/sh
# git-notes auto-sync pull
git-notes sync pull
"#;

    let pre_push_script = r#"#!/bin/sh
# git-notes auto-sync push
git-notes sync push
"#;

    fs::write(&post_merge_path, post_merge_script)?;
    fs::write(&pre_push_path, pre_push_script)?;

    // Make executable on unix-like
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&post_merge_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&post_merge_path, perms.clone())?;
        fs::set_permissions(&pre_push_path, perms)?;
    }

    Ok(())
}

pub fn uninstall_hooks(repo_path: &Path) -> Result<()> {
    let hooks_dir = repo_path.join(".git").join("hooks");

    let post_merge_path = hooks_dir.join("post-merge");
    if post_merge_path.exists() {
        fs::remove_file(post_merge_path)?;
    }

    let pre_push_path = hooks_dir.join("pre-push");
    if pre_push_path.exists() {
        fs::remove_file(pre_push_path)?;
    }

    Ok(())
}
