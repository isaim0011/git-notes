from pathlib import Path
import pytest

from gn_hooks.install import find_git_root


def test_find_git_root_at_repo_root(tmp_path: Path):
    """find_git_root returns the path itself when .git dir is directly in path."""
    git_dir = tmp_path / ".git"
    git_dir.mkdir()

    result = find_git_root(tmp_path)
    assert result == tmp_path.resolve()


def test_find_git_root_from_nested_subdir(tmp_path: Path):
    """find_git_root walks up directory tree from a nested subdirectory to find .git dir."""
    git_dir = tmp_path / ".git"
    git_dir.mkdir()

    nested_dir = tmp_path / "sub1" / "sub2" / "sub3"
    nested_dir.mkdir(parents=True)

    result = find_git_root(nested_dir)
    assert result == tmp_path.resolve()


def test_find_git_root_from_file_path(tmp_path: Path):
    """find_git_root works when passed a file path inside a repository."""
    git_dir = tmp_path / ".git"
    git_dir.mkdir()

    sub_dir = tmp_path / "src"
    sub_dir.mkdir()
    file_path = sub_dir / "main.py"
    file_path.write_text("print('hello')")

    result = find_git_root(file_path)
    assert result == tmp_path.resolve()


def test_find_git_root_with_relative_path(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """find_git_root handles relative paths correctly by resolving them."""
    git_dir = tmp_path / ".git"
    git_dir.mkdir()

    sub_dir = tmp_path / "sub"
    sub_dir.mkdir()

    monkeypatch.chdir(sub_dir)
    result = find_git_root(Path("."))
    assert result == tmp_path.resolve()


def test_find_git_root_when_git_is_a_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """find_git_root ignores .git if it is a file instead of a directory."""
    resolved_tmp = tmp_path.resolve()
    sub_dir = resolved_tmp / "sub"
    sub_dir.mkdir()
    git_file = sub_dir / ".git"
    git_file.write_text("gitdir: /path/to/git")

    # Stop search loop at resolved_tmp to isolate test from outer directory tree
    original_parent_fget = Path.parent.fget

    def custom_parent(self):
        if self == resolved_tmp:
            return self
        return original_parent_fget(self)

    monkeypatch.setattr(Path, "parent", property(custom_parent))

    with pytest.raises(FileNotFoundError, match="Not a git repository"):
        find_git_root(sub_dir)


def test_find_git_root_not_found_raises_file_not_found_error(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """find_git_root raises FileNotFoundError when no parent directory contains a .git directory."""
    resolved_tmp = tmp_path.resolve()
    no_git_dir = resolved_tmp / "no_git_dir"
    no_git_dir.mkdir()

    # Stop search loop at resolved_tmp to guarantee clean test isolation
    original_parent_fget = Path.parent.fget

    def custom_parent(self):
        if self == resolved_tmp:
            return self
        return original_parent_fget(self)

    monkeypatch.setattr(Path, "parent", property(custom_parent))

    with pytest.raises(FileNotFoundError, match="Not a git repository"):
        find_git_root(no_git_dir)
