# git-notes

> **Decentralized code comments — history-safe, namespace-scoped, sync anywhere.**

[![CI](https://github.com/isaim0011/git-notes/actions/workflows/ci.yml/badge.svg)](https://github.com/isaim0011/git-notes/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/isaim0011/git-notes?color=brightgreen)](https://github.com/isaim0011/git-notes/releases)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](./LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen)](https://github.com/isaim0011/git-notes/blob/main/.github/pull_request_template.md)
[![GitHub Stars](https://img.shields.io/github/stars/isaim0011/git-notes?style=social)](https://github.com/isaim0011/git-notes/stargazers)
[![Ko-fi](https://img.shields.io/badge/Support-Ko--fi-FF5E5B?logo=ko-fi&logoColor=white)](https://ko-fi.com/isaim0011)

`git-notes` is a polyglot open-source platform that brings persistent, decentralized code comments to **every surface** — without ever touching your commit history.

<p align="center">
  <img src="./assets/demo.svg" alt="git-notes Terminal Demo" width="100%" />
</p>

---

## Features

| Feature | Description |
|---|---|
| 🔒 **Zero history rewrite** | Notes live in `refs/notes/*`, never in commits |
| 🌐 **Works everywhere** | CLI · TUI · VS Code · Chrome · Web · CI |
| 📦 **Offline-first** | Notes live in the repo — no server needed |
| 💬 **Threaded replies** | Reply directly to existing notes in-tree |
| 🔀 **Smart merge** | Union + LWW strategy on sync conflicts |
| 🔌 **Pluggable** | Add namespaces, merge strategies, surfaces |
| 🐙 **GitHub Bridge** | Sync notes ↔ PR review comments |

---

## Installation

### ⚡ Quick Install (Prebuilt Binaries)

Download standalone binaries directly from [GitHub Releases v0.1.0](https://github.com/isaim0011/git-notes/releases/tag/v0.1.0):

| Platform | Binary | One-line Command |
|---|---|---|
| **Windows** | [`git-notes.exe`](https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-windows-x86_64.exe) | `Invoke-WebRequest -Uri "https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-windows-x86_64.exe" -OutFile git-notes.exe` |
| **Linux (x86_64)** | [`git-notes`](https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-linux-x86_64) | `curl -L https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-linux-x86_64 -o git-notes && chmod +x git-notes` |
| **macOS (Apple Silicon)** | [`git-notes`](https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-macos-aarch64) | `curl -L https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-macos-aarch64 -o git-notes && chmod +x git-notes` |
| **macOS (Intel)** | [`git-notes`](https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-macos-x86_64) | `curl -L https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-macos-x86_64 -o git-notes && chmod +x git-notes` |

### Build from Source (Rust)
```bash
git clone https://github.com/isaim0011/git-notes.git
cd git-notes
cargo build --release
```

### VS Code Extension
Download from [Releases](https://github.com/isaim0011/git-notes/releases/tag/v0.1.0) or run `bun run build` in `packages/vscode-ext`.

### Chrome Extension
Download [`git-notes-chrome.zip`](https://github.com/isaim0011/git-notes/releases/download/v0.1.0/git-notes-chrome.zip) from Releases and load unpacked into Chrome (`chrome://extensions`).

### Python Hooks (auto-sync on push/pull)
```bash
cd python/gn_hooks
pip install -e .
git-notes-hooks install
```

---

## Quick Start

```bash
# Add a comment on line 42 of main.rs
git-notes add -f src/main.rs -l 42 -m "Why is this O(n²)?"

# Reply to an existing note thread
git-notes reply a1b2c3 -m "Fixed in commit abc123!"

# Show a specific note and its full reply thread
git-notes show a1b2c3 --thread

# List all review notes
git-notes list --namespace review

# Sync to remote
git-notes sync push

# Mark resolved
git-notes resolve a1b2c3 --status approved

# Launch TUI browser
git-notes-tui

# Export static HTML viewer
git-notes export --format html --out ./docs/notes/
```

---

## Namespaces

| Ref | Purpose |
|---|---|
| `refs/notes/comments` | General code discussions |
| `refs/notes/review` | Formal review approvals / rejections |
| `refs/notes/todos` | Tracked TODO items |
| `refs/notes/ci` | *(extensible)* CI annotations |
| `refs/notes/security` | *(extensible)* Security findings |

---

## Architecture

```
SURFACES                  CORE ENGINE               DISTRIBUTION
────────                  ───────────               ────────────
CLI (Rust/clap)  ──┐
TUI (Rust/ratatui)─┤     ┌─────────────┐     ┌──────────────────┐
VS Code (TS/Bun) ──┼────▶│  gn-core    │────▶│   Sync Layer     │────▶ Remote
Chrome Ext (TS)  ──┤     │  (gix-based)│     │ union+LWW merge  │
Web Viewer (Vite)──┘     └──────┬──────┘     └────────┬─────────┘
                                │                      │
                         Namespaces              GitHub Bridge
                    refs/notes/comments          (Go service)
                    refs/notes/review       notes ↔ PR comments
                    refs/notes/todos
```

---

## Monorepo Structure

```
git-notes/
├── crates/
│   ├── gn-core/        # Core engine (pure Rust, gix)
│   ├── gn-cli/         # CLI binary (clap v4)
│   ├── gn-tui/         # TUI (ratatui + crossterm)
│   └── gn-sync/        # Sync / merge layer
├── packages/
│   ├── vscode-ext/     # VS Code extension (TypeScript)
│   ├── chrome-ext/     # Chrome MV3 extension (TypeScript)
│   └── web-viewer/     # Static HTML export (Vite + Preact)
├── services/
│   └── github-bridge/  # GitHub Action + daemon (Go)
├── python/
│   └── gn_hooks/       # Git hooks helper (pip)
└── .github/workflows/  # CI + Release
```

---

## Contributing

This project uses:
- **Rust** 1.70+ for core crates
- **Bun** 1.x for TypeScript packages
- **Go** 1.21+ for services
- **Python** 3.11+ for hooks

```bash
git clone https://github.com/git-notes/git-notes
cd git-notes

# Rust
cargo build --workspace
cargo test --workspace

# TypeScript
bun install
bun run build

# Go
go work sync
go test ./...

# Python
cd python/gn_hooks && pip install -e ".[dev]"
```

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
