# git-notes

> **Decentralized code comments — history-safe, namespace-scoped, sync anywhere.**

[![CI](https://github.com/git-notes/git-notes/actions/workflows/ci.yml/badge.svg)](https://github.com/git-notes/git-notes/actions)
[![Crates.io](https://img.shields.io/crates/v/git-notes-cli)](https://crates.io/crates/git-notes-cli)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

`git-notes` is a polyglot open-source platform that brings persistent, decentralized code comments to **every surface** — without ever touching your commit history.

```
┌──────────────┬───────────────────────────┬────────────────────┐
│ File Tree    │  Diff Hunk                │  Comments          │
│              │  ● line 42 [2 comments]   │  > Alice: why O²?  │
│ src/         │   fn process(items) {     │  > Bob: fixed in   │
│   main.rs●   │ > let mut map = ...       │    next commit     │
│   lib.rs     │                           │  [Reply] [Resolve] │
└──────────────┴───────────────────────────┴────────────────────┘
```

---

## Features

| Feature | Description |
|---|---|
| 🔒 **Zero history rewrite** | Notes live in `refs/notes/*`, never in commits |
| 🌐 **Works everywhere** | CLI · TUI · VS Code · Chrome · Web · CI |
| 📦 **Offline-first** | Notes live in the repo — no server needed |
| 🔀 **Smart merge** | Union + LWW strategy on sync conflicts |
| 🔌 **Pluggable** | Add namespaces, merge strategies, surfaces |
| 🐙 **GitHub Bridge** | Sync notes ↔ PR review comments |

---

## Installation

### CLI + TUI (Rust)
```bash
cargo install git-notes-cli git-notes-tui
```

### VS Code Extension
```
ext install git-notes.vscode-git-notes
```

### Chrome Extension
Download from Chrome Web Store or load `packages/chrome-ext/dist` unpacked.

### Python hooks (auto-sync on push/pull)
```bash
pip install git-notes-hooks
git-notes-hooks install
```

---

## Quick Start

```bash
# Add a comment on line 42 of main.rs
git-notes add -f src/main.rs -l 42 -m "Why is this O(n²)?"

# List all review notes
git-notes list --namespace review

# Show a specific note
git-notes show a1b2c3

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
