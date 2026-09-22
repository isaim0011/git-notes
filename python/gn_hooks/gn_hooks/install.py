import os
import shutil
from pathlib import Path
from typing import Dict, Any

HOOKS_DIR = Path(__file__).parent / "hooks"
SUPPORTED_HOOKS = ["post_merge.sh", "pre_push.sh", "post_commit.sh"]

def find_git_root(path: Path) -> Path:
    """Walks up the directory tree to find the .git directory."""
    current = path.resolve()
    while current != current.parent:
        if (current / ".git").is_dir():
            return current
        current = current.parent
    raise FileNotFoundError("Not a git repository (or any of the parent directories).")

def _get_hook_dst(repo_path: Path, hook_name: str) -> Path:
    base_name = hook_name.replace(".sh", "").replace("_", "-")
    return repo_path / ".git" / "hooks" / base_name

def install_hooks(repo_path: Path, binary: str = "git-notes") -> Dict[str, bool]:
    """Writes all hook scripts, returns {hook_name: success}."""
    results = {}
    for hook in SUPPORTED_HOOKS:
        src = HOOKS_DIR / hook
        if not src.exists():
            continue
            
        dst = _get_hook_dst(repo_path, hook)
        try:
            # Simple install: just copy our script.
            # In a more advanced version, we could append to existing scripts.
            shutil.copy2(src, dst)
            os.chmod(dst, 0o755)
            results[hook] = True
        except Exception:
            results[hook] = False
    return results

def uninstall_hooks(repo_path: Path) -> Dict[str, bool]:
    """Removes git-notes hooks."""
    results = {}
    for hook in SUPPORTED_HOOKS:
        dst = _get_hook_dst(repo_path, hook)
        if dst.exists():
            try:
                # Check if it's our hook (contains git-notes)
                content = dst.read_text(encoding="utf-8")
                if "git-notes" in content:
                    dst.unlink()
                    results[hook] = True
                else:
                    results[hook] = False
            except Exception:
                results[hook] = False
        else:
            results[hook] = False
    return results

def get_hook_status(repo_path: Path) -> Dict[str, Dict[str, bool]]:
    """Checks if hooks exist and are ours."""
    results = {}
    for hook in SUPPORTED_HOOKS:
        dst = _get_hook_dst(repo_path, hook)
        exists = dst.exists()
        installed = False
        if exists:
            try:
                content = dst.read_text(encoding="utf-8")
                if "git-notes hook" in content:
                    installed = True
            except Exception:
                pass
        results[hook] = {"exists": exists, "installed": installed}
    return results
