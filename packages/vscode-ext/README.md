# git-notes for Visual Studio Code & Cursor

<p align="center">
  <img src="https://raw.githubusercontent.com/isaim0011/git-notes/main/packages/chrome-ext/icon128.png" width="96" height="96" alt="git-notes logo" />
</p>

<p align="center">
  <strong>Decentralized, vendor-neutral code reviews and line annotations stored directly in your Git repository.</strong>
</p>

<p align="center">
  <a href="https://open-vsx.org/extension/isaim0011/vscode-git-notes"><img src="https://img.shields.io/open-vsx/v/isaim0011/vscode-git-notes?color=purple&label=Open%20VSX" alt="Open VSX" /></a>
  <a href="https://github.com/isaim0011/git-notes"><img src="https://img.shields.io/github/v/release/isaim0011/git-notes?color=brightgreen&label=GitHub" alt="GitHub" /></a>
  <a href="https://crates.io/crates/gn-cli"><img src="https://img.shields.io/crates/v/gn-cli?color=orange&label=crates.io" alt="crates.io" /></a>
  <a href="https://pypi.org/project/git-notes-hooks/"><img src="https://img.shields.io/pypi/v/git-notes-hooks?color=blue&label=PyPI" alt="PyPI" /></a>
  <img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg" alt="License" />
</p>

---

## ⚡ What is git-notes?

`git-notes` enables permanent, decentralized code reviews and contextual comments without vendor lock-in. Instead of being locked inside GitHub PR databases or GitLab issues, all notes, comment threads, reviews, and resolution states are content-addressed and synchronized using standard Git refs (`refs/notes/*`).

This extension integrates `git-notes` directly into your editor with editor gutter icons, inlay hints, contextual menus, and an interactive thread webview.

---

## ✨ Features

- 📌 **Editor Gutter Annotations**: Visually indicates lines with open comments, pending reviews, or unresolved discussion threads.
- 💡 **Inlay Hints**: View note authors, timestamps, and thread snippets inline alongside your code.
- 💬 **Interactive Discussion Panel**: View the entire threaded conversation, reply to comments, approve, reject, or mark notes as resolved.
- 🔄 **Automatic Synchronization**: Automatically fetches and merges notes when switching branches, opening projects, or saving files.
- ⚡ **Zero Vendor Lock-in**: All discussion data lives right inside your `.git` folder and pushes/pulls with your standard Git remote (`origin`).
- 🛠️ **Seamless CLI Integration**: Powered by the ultra-fast Rust `git-notes` CLI.

---

## 🚀 Quick Start

### 1. Prerequisites
Ensure you have the `git-notes` CLI binary installed on your PATH:
```bash
# Via Cargo (cross-platform):
cargo install gn-cli --bin git-notes

# Or download prebuilt binaries from GitHub Releases:
# https://github.com/isaim0011/git-notes/releases
```

### 2. Adding a Note
1. Open any file in your Git repository.
2. Right-click on any line and select **`git-notes: Add Note`** (or press `Ctrl+Shift+P` / `Cmd+Shift+P` → type `git-notes: Add Note`).
3. Type your comment and press **Enter**. A note is created and anchored to that exact line and commit!

### 3. Viewing & Replying to Threads
- Click any gutter annotation, or right-click and select **`git-notes: Show Note Thread`**.
- The rich thread webview allows you to review the discussion, reply to teammates, or resolve issues.

---

## ⚙️ Configuration Settings

Customize `git-notes` to match your team's workflow in **Settings** (`Ctrl+,` or `Cmd+,`):

| Setting | Default | Description |
|---|---|---|
| `git-notes.binaryPath` | `"git-notes"` | Absolute path or PATH executable name for the `git-notes` CLI. |
| `git-notes.autoSync` | `false` | Automatically pull and push notes on file save and periodic intervals. |
| `git-notes.defaultNamespace` | `"comments"` | Default namespace for new notes (`comments`, `review`, `todos`). |

---

## ⌨️ Commands

| Command | Title | Context |
|---|---|---|
| `git-notes.add` | `git-notes: Add Note` | Editor Context Menu & Command Palette |
| `git-notes.showPanel` | `git-notes: Show Note Thread` | Editor Context Menu & Command Palette |
| `git-notes.list` | `git-notes: List Notes` | Command Palette |
| `git-notes.sync` | `git-notes: Sync Notes` | Command Palette |

---

## 🔗 Ecosystem

- **Rust Core & CLI**: [crates.io/crates/gn-cli](https://crates.io/crates/gn-cli)
- **Git Hooks**: [pypi.org/project/git-notes-hooks](https://pypi.org/project/git-notes-hooks/)
- **Chrome Extension**: [GitHub Releases](https://github.com/isaim0011/git-notes/releases)
- **Documentation & Source**: [github.com/isaim0011/git-notes](https://github.com/isaim0011/git-notes)

---

## 📄 License

Dual-licensed under either **MIT** or **Apache 2.0** at your option.
