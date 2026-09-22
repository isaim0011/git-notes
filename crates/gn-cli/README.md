# git-notes-cli (`git-notes`)

> **Command-line interface for `git-notes` — decentralized code comments that live inside Git forever.**

[![Crates.io](https://img.shields.io/crates/v/gn-cli.svg)](https://crates.io/crates/gn-cli)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/isaim0011/git-notes)

`git-notes` allows developers to add comments, reviews, and discussion threads directly to commits, files, and line numbers without altering Git commit history.

---

## ⚡ Installation

Install from crates.io via Cargo:

```bash
cargo install gn-cli --bin git-notes
```

Or download precompiled standalone binaries from [GitHub Releases](https://github.com/isaim0011/git-notes/releases).

---

## 🚀 Commands

### Add a Note
```bash
git-notes add -f src/main.rs -l 42 -m "Why is this O(n²)?"
```

### Reply to a Thread
```bash
git-notes reply 72818dac -m "Refactored to HashMap lookup!"
```

### Show Discussion Thread
```bash
git-notes show 72818dac --thread
```

### List Notes
```bash
git-notes list --namespace comments
git-notes list --namespace review
```

### Resolve / Approve
```bash
git-notes resolve 72818dac --status approved
```

### Export Notes
```bash
git-notes export --format markdown   # outputs NOTES.md
git-notes export --format json       # outputs notes.json
git-notes export --format html       # outputs index.html
```

---

## 🔗 Repository

- **GitHub**: [https://github.com/isaim0011/git-notes](https://github.com/isaim0011/git-notes)
- **Releases**: [https://github.com/isaim0011/git-notes/releases](https://github.com/isaim0011/git-notes/releases)
