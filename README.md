<div align="center">

<img src="https://raw.githubusercontent.com/isaim0011/git-notes/main/assets/icon.svg" width="80" height="80" alt="git-notes logo" />

# git-notes

**Decentralized code reviews and annotations stored directly in Git.**  
No database. No vendor. No history rewrites. Just `refs/notes/*`.

[![CI](https://github.com/isaim0011/git-notes/actions/workflows/ci.yml/badge.svg)](https://github.com/isaim0011/git-notes/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/isaim0011/git-notes?color=brightgreen&label=Release)](https://github.com/isaim0011/git-notes/releases)
[![Crates.io](https://img.shields.io/crates/v/gn-cli?label=crates.io&color=orange)](https://crates.io/crates/gn-cli)
[![PyPI](https://img.shields.io/pypi/v/git-notes-hooks?label=PyPI&color=blue)](https://pypi.org/project/git-notes-hooks/)
[![Open VSX](https://img.shields.io/badge/Open%20VSX-v0.1.2-purple)](https://open-vsx.org/extension/isaim0011/vscode-git-notes)
[![GitHub Action](https://img.shields.io/badge/GitHub_Action-Marketplace-2088FF?logo=github-actions&logoColor=white)](https://github.com/marketplace/actions/git-notes-sync)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](./LICENSE-MIT)
[![Ko-fi](https://img.shields.io/badge/Support-Ko--fi-FF5E5B?logo=ko-fi&logoColor=white)](https://ko-fi.com/isaim0011)

</div>

---

<p align="center">
  <img src="https://raw.githubusercontent.com/isaim0011/git-notes/main/assets/demo.gif" alt="git-notes Terminal Demo — add → TUI → sync → GitHub PR" width="100%" />
</p>

---

## Why git-notes?

Code review comments die in GitHub. PRs close. Context vanishes. git-notes keeps every discussion **inside the repository** — synced via standard Git, readable in your editor, terminal, and browser.

| The problem | git-notes solves it by |
|---|---|
| PR comments disappear after merge | Notes live in `refs/notes/*` — permanent, versioned |
| Review context is vendor-locked | Portable across GitHub, GitLab, Gitea, Forgejo |
| No offline access to discussions | Notes clone with the repo — zero network required |
| Context switches between IDE and browser | Inline annotations in VS Code, TUI, Chrome, `git blame` |
| Team knowledge dies with the PR | Notes sync via `git push` / `git fetch` like any ref |

---

## Install in 10 seconds

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/isaim0011/git-notes/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/isaim0011/git-notes/main/install.ps1 | iex
```

**Homebrew (macOS / Linux):**
```bash
brew install isaim0011/tap/git-notes
```

**Scoop (Windows):**
```powershell
scoop bucket add git-notes-bucket https://github.com/isaim0011/scoop-bucket
scoop install git-notes
```

**Cargo:**
```bash
cargo install gn-cli --bin git-notes
```

→ [See all platforms & download binaries](https://github.com/isaim0011/git-notes/releases)

---

## Quick Start

```bash
# 1. Add a note anchored to line 42 of an auth file
git-notes add -f src/auth.rs -l 42 -m "Validate JWT expiry before decoding claims"

# 2. Check repo health (hooks, refs, stale notes)
git-notes doctor

# 3. Browse notes in the interactive TUI
git-notes-tui

# 4. See notes inline in git blame
git-notes blame --file src/auth.rs

# 5. Pull in a team's existing GitHub PR review comments instantly
git-notes import-pr --pr 42

# 6. Sync bidirectionally with origin
git-notes sync push && git-notes sync pull

# 7. Get an AI summary of all open threads before a release
git-notes summarize --namespace review

# 8. List, filter, resolve
git-notes list --namespace review --status open
git-notes resolve a1b2c3 --status approved
```

---

## Features

<table>
<tr>
<td width="50%">

### 🔒 Permanent & Portable
Notes live in `refs/notes/*` — never touch your commit history. Clone the repo, get the discussions.

### ⚡ Stale-Note Detection
`git-notes doctor` warns when commented code has moved or been deleted since the note was anchored. **Nobody else does this.**

### 🤖 AI Thread Summarizer
`git-notes summarize` generates an LLM digest of all open discussion threads — the "what's blocking release?" command.

### 🔀 Conflict-Free Merge
Union + Last-Write-Wins strategy means concurrent note editing never blocks your `git pull`.

</td>
<td width="50%">

### 🌐 Every Surface
CLI → TUI → VS Code → Chrome Extension → GitHub PR comments — one note, everywhere.

### 🐙 GitHub Action CI Loop
Install [`isaim0011/git-notes`](https://github.com/marketplace/actions/git-notes-sync) in any repo — auto-posts notes as inline PR review comments.

### 📋 PR Comment Importer
`git-notes import-pr --pr 42` pulls your existing GitHub PR discussions into notes in seconds.

### 💬 Threaded Discussions
Reply, approve, reject, resolve — full conversation trees anchored to specific lines.

</td>
</tr>
</table>

---

## GitHub Action — CI Viral Loop

Every repo that installs this action links back to git-notes:

```yaml
# .github/workflows/git-notes.yml
name: Sync git-notes
on: [pull_request, push]
jobs:
  sync:
    runs-on: ubuntu-latest
    permissions:
      pull-requests: write
      contents: read
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - uses: isaim0011/git-notes@v0.1.0
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          namespace: comments
```

**What it does:** Downloads the CLI, fetches `refs/notes/*`, and posts open notes as inline PR review comments — automatically. [→ GitHub Marketplace](https://github.com/marketplace/actions/git-notes-sync)

---

## Architecture

```
SURFACES                   CORE                     DISTRIBUTION
────────                   ────                     ────────────
CLI  (Rust/clap)  ─┐
TUI  (ratatui)    ─┤   ┌───────────┐           ┌────────────────┐
VS Code (TypeScript)┼──▶│  gn-core  │──────────▶│  Sync Layer    │──▶ git remote
Chrome  (TypeScript)┤   │  (gix)    │           │  union + LWW   │
Web Viewer (Vite)  ─┘   └─────┬─────┘           └───────┬────────┘
                               │                         │
                          refs/notes/*            GitHub Bridge
                          /comments               (Go daemon)
                          /review           notes ↔ PR comments
                          /todos
```

---

## Ecosystem

| Package | Registry | Install |
|---|---|---|
| `gn-cli` (CLI binary) | [crates.io](https://crates.io/crates/gn-cli) | `cargo install gn-cli --bin git-notes` |
| `gn-tui` (TUI browser) | [crates.io](https://crates.io/crates/gn-tui) | `cargo install gn-tui --bin git-notes-tui` |
| `gn-core` (Rust library) | [crates.io](https://crates.io/crates/gn-core) | `cargo add gn-core` |
| `git-notes-hooks` (Python) | [PyPI](https://pypi.org/project/git-notes-hooks/) | `pip install git-notes-hooks` |
| VS Code Extension | [Open VSX](https://open-vsx.org/extension/isaim0011/vscode-git-notes) | `code --install-extension isaim0011.vscode-git-notes` |
| GitHub Action | [Marketplace](https://github.com/marketplace/actions/git-notes-sync) | `uses: isaim0011/git-notes@v0.1.0` |
| Docker (bridge daemon) | [GHCR](https://github.com/isaim0011/git-notes/pkgs/container/github-bridge) | `docker pull ghcr.io/isaim0011/git-notes/github-bridge:latest` |

---

## Monorepo Structure

```
git-notes/
├── crates/
│   ├── gn-core/        # Core engine — reads/writes git note objects (gix)
│   ├── gn-cli/         # CLI binary — clap v4, all subcommands
│   ├── gn-tui/         # TUI browser — ratatui + syntect syntax highlighting
│   └── gn-sync/        # Sync/merge layer — union + LWW strategies
├── packages/
│   ├── vscode-ext/     # VS Code & Cursor extension (TypeScript/Bun)
│   ├── chrome-ext/     # Chrome MV3 extension (TypeScript)
│   ├── homebrew/       # Homebrew formula
│   └── scoop/          # Windows Scoop manifest
├── services/
│   └── github-bridge/  # HTTP bridge daemon — notes ↔ GitHub PR (Go)
├── python/
│   └── gn_hooks/       # Auto-sync git hooks (pip install)
├── action.yml          # GitHub Marketplace action
├── install.sh          # Universal Linux/macOS installer
└── install.ps1         # Universal Windows installer
```

---

## Contributing

```bash
git clone https://github.com/isaim0011/git-notes.git
cd git-notes

# Rust (CLI + TUI + core)
cargo build --workspace
cargo test --workspace

# TypeScript (VS Code + Chrome extensions)
bun install && bun run build

# Go (GitHub bridge daemon)
go work sync && go test ./...

# Python (git hooks)
pip install -e "python/gn_hooks[dev]"
```

Stack: **Rust** 1.70+ · **Bun** 1.x · **Go** 1.21+ · **Python** 3.11+  
PRs welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md) and [open issues](https://github.com/isaim0011/git-notes/issues).

---

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.

<div align="center">
  <sub>Built with ❤️ by <a href="https://github.com/isaim0011">Bimo</a> — <a href="https://ko-fi.com/isaim0011">Support on Ko-fi</a></sub>
</div>
