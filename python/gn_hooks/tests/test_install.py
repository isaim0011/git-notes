import os
import shutil
import pytest
from pathlib import Path
from gn_hooks.install import (
    find_git_root,
    _get_hook_dst,
    install_hooks,
    uninstall_hooks,
    get_hook_status,
    SUPPORTED_HOOKS,
)


@pytest.fixture
def temp_git_repo(tmp_path: Path) -> Path:
    """Creates a temporary git repository structure with a .git/hooks directory."""
    repo = tmp_path / "test_repo"
    hooks_dir = repo / ".git" / "hooks"
    hooks_dir.mkdir(parents=True, exist_ok=True)
    return repo


def test_find_git_root_success(tmp_path: Path):
    repo = tmp_path / "my_project"
    git_dir = repo / ".git"
    git_dir.mkdir(parents=True)

    deep_dir = repo / "a" / "b" / "c"
    deep_dir.mkdir(parents=True)

    found_root = find_git_root(deep_dir)
    assert found_root == repo.resolve()


def test_find_git_root_not_found(tmp_path: Path):
    non_git_dir = tmp_path / "no_git"
    non_git_dir.mkdir(parents=True)

    with pytest.raises(FileNotFoundError, match="Not a git repository"):
        find_git_root(non_git_dir)


def test_get_hook_dst(temp_git_repo: Path):
    dst_merge = _get_hook_dst(temp_git_repo, "post_merge.sh")
    assert dst_merge == temp_git_repo / ".git" / "hooks" / "post-merge"

    dst_push = _get_hook_dst(temp_git_repo, "pre_push.sh")
    assert dst_push == temp_git_repo / ".git" / "hooks" / "pre-push"

    dst_commit = _get_hook_dst(temp_git_repo, "post_commit.sh")
    assert dst_commit == temp_git_repo / ".git" / "hooks" / "post-commit"


def test_install_hooks_success(temp_git_repo: Path):
    results = install_hooks(temp_git_repo)

    for hook in SUPPORTED_HOOKS:
        assert results.get(hook) is True
        dst = _get_hook_dst(temp_git_repo, hook)
        assert dst.exists()
        assert os.access(dst, os.X_OK)


def test_install_hooks_missing_source(temp_git_repo: Path, monkeypatch: pytest.MonkeyPatch):
    empty_dir = temp_git_repo / "empty_hooks_src"
    empty_dir.mkdir()
    monkeypatch.setattr("gn_hooks.install.HOOKS_DIR", empty_dir)

    results = install_hooks(temp_git_repo)
    assert results == {}


def test_install_hooks_copy_error(temp_git_repo: Path, monkeypatch: pytest.MonkeyPatch):
    def mock_copy2(*args, **kwargs):
        raise OSError("Permission denied")

    monkeypatch.setattr(shutil, "copy2", mock_copy2)
    results = install_hooks(temp_git_repo)

    for hook in SUPPORTED_HOOKS:
        assert results[hook] is False


def test_uninstall_hooks_success(temp_git_repo: Path):
    install_hooks(temp_git_repo)
    results = uninstall_hooks(temp_git_repo)

    for hook in SUPPORTED_HOOKS:
        assert results[hook] is True
        dst = _get_hook_dst(temp_git_repo, hook)
        assert not dst.exists()


def test_uninstall_hooks_preserves_custom_hook(temp_git_repo: Path):
    custom_dst = _get_hook_dst(temp_git_repo, "post_merge.sh")
    custom_dst.write_text("#!/bin/sh\necho custom script", encoding="utf-8")

    results = uninstall_hooks(temp_git_repo)

    assert results["post_merge.sh"] is False
    assert custom_dst.exists()


def test_uninstall_hooks_missing_hook(temp_git_repo: Path):
    results = uninstall_hooks(temp_git_repo)

    for hook in SUPPORTED_HOOKS:
        assert results[hook] is False


def test_uninstall_hooks_unlink_exception(temp_git_repo: Path, monkeypatch: pytest.MonkeyPatch):
    install_hooks(temp_git_repo)

    original_unlink = Path.unlink

    def mock_unlink(self, *args, **kwargs):
        if "post-merge" in str(self):
            raise OSError("Cannot delete")
        return original_unlink(self, *args, **kwargs)

    monkeypatch.setattr(Path, "unlink", mock_unlink)

    results = uninstall_hooks(temp_git_repo)
    assert results["post_merge.sh"] is False


def test_get_hook_status(temp_git_repo: Path):
    # Installed git-notes hook
    installed_dst = _get_hook_dst(temp_git_repo, "post_merge.sh")
    installed_dst.write_text("#!/bin/sh\n# git-notes hook\ngit notes sync", encoding="utf-8")

    # Custom hook (exists but not ours)
    custom_dst = _get_hook_dst(temp_git_repo, "pre_push.sh")
    custom_dst.write_text("#!/bin/sh\necho custom", encoding="utf-8")

    status = get_hook_status(temp_git_repo)

    assert status["post_merge.sh"] == {"exists": True, "installed": True}
    assert status["pre_push.sh"] == {"exists": True, "installed": False}
    assert status["post_commit.sh"] == {"exists": False, "installed": False}


def test_get_hook_status_read_exception(temp_git_repo: Path, monkeypatch: pytest.MonkeyPatch):
    dst = _get_hook_dst(temp_git_repo, "post_merge.sh")
    dst.write_text("#!/bin/sh\n# git-notes hook", encoding="utf-8")

    def mock_read_text(self, *args, **kwargs):
        raise OSError("Read failure")

    monkeypatch.setattr(Path, "read_text", mock_read_text)

    status = get_hook_status(temp_git_repo)
    assert status["post_merge.sh"] == {"exists": True, "installed": False}
