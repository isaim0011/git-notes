# git-notes-hooks

> **Automatic Git hooks to keep `refs/notes/*` seamlessly synchronized across all remotes on every push and pull.**

[![PyPI Version](https://img.shields.io/pypi/v/git-notes-hooks.svg)](https://pypi.org/project/git-notes-hooks/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/isaim0011/git-notes)
[![Python Versions](https://img.shields.io/pypi/pyversions/git-notes-hooks.svg)](https://pypi.org/project/git-notes-hooks/)

`git-notes-hooks` provides automated git lifecycle integration for the **[git-notes](https://github.com/isaim0011/git-notes)** ecosystem. It installs zero-friction client-side git hooks into your repositories to ensure decentralized code review comments and notes travel with your commits automatically.

---

## ⚡ Quick Installation

Install from PyPI via pip:

```bash
pip install git-notes-hooks
```

Or using pipx:

```bash
pipx install git-notes-hooks
```

---

## 🚀 Usage

Navigate to any Git repository and run:

### 1. Install Hooks
Install the automated push/pull synchronization hooks into `.git/hooks`:

```bash
git-notes-hooks install
```

This installs:
- `post-commit`: Updates local note indices on every commit
- `pre-push`: Pushes `refs/notes/*` to `origin` whenever you run `git push`
- `post-merge`: Fetches and merges remote notes from `origin` on `git pull`

### 2. Check Hook Status
Verify active git-notes hooks in your current repository:

```bash
git-notes-hooks status
```

### 3. Uninstall Hooks
Safely remove git-notes hooks at any time:

```bash
git-notes-hooks uninstall
```

---

## 🔒 History-Safe & Non-Destructive

`git-notes-hooks` operates strictly on Git's native note references (`refs/notes/comments`, `refs/notes/review`, `refs/notes/todos`). It **never modifies commit SHAs, git history, or working tree files**.

---

## 🔗 Links & Resources

- **GitHub Repository**: [https://github.com/isaim0011/git-notes](https://github.com/isaim0011/git-notes)
- **Issue Tracker**: [https://github.com/isaim0011/git-notes/issues](https://github.com/isaim0011/git-notes/issues)
- **Documentation**: [https://github.com/isaim0011/git-notes/blob/main/docs/ARCHITECTURE.md](https://github.com/isaim0011/git-notes/blob/main/docs/ARCHITECTURE.md)
- **Release Downloads**: [https://github.com/isaim0011/git-notes/releases](https://github.com/isaim0011/git-notes/releases)

---

## 📄 License

Licensed under either of:
- Apache License, Version 2.0
- MIT License
at your option.
