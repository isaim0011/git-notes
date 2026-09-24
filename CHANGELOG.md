# Changelog

All notable changes to the `git-notes` ecosystem are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.7] - 2026-09-24

### Added
- **`gn shortcuts` command**: Built an interactive cheat sheet and configurator for quickies, abbreviations, and keybindings.
- **Custom Shortcut Engine**: Added `--set <alias>=<command>` (e.g. `gn shortcuts --set c="add -m"`) to persist custom user shortcuts in `~/.git-notes/shortcuts.json`. Added `--reset` to restore defaults.
- **VS Code / Cursor Default Keybindings**:
  - `Alt+N` (`Cmd+Option+N` on macOS): Add note to focused editor line.
  - `Alt+Shift+N` (`Cmd+Option+Shift+N` on macOS): Open note discussion thread panel.
  - `Alt+S` (`Cmd+Option+S` on macOS): Trigger bidirectional note sync with remote.
- **TUI Keyboard Navigation**: Documented and mapped `↑`/`↓`/`k`/`j` for navigation, `Enter`/`→` for expanding panels, `r` for quick inline replies, and `ok`/`a` for resolving/approving notes.
- **Comprehensive Docs**: Documented CLI quickies (`gn a`, `gn l`, `gn r`, `gn ok`, `gn s`, `gn doc`) across all root and sub-crate READMEs.

### Fixed
- Replaced static version badge URLs with dynamic query shields across all package and sub-crate READMEs.

---

## [0.1.6] - 2026-09-24

### Added
- **Adaptive User Learning Algorithm (`learning.rs`)**: Smart frequency profiling that learns from usage patterns and suggests proactive shortcuts (`gn l`, `gn d`, etc.).
- **Interactive Terminal Note Picker (`gn s`)**: Arrow-key navigable thread selector with instant previews.
- **Compact Numbered Index Table (`gn l`)**: Displays `[1]`, `[2]`, `[3]` indices allowing quick replies (`gn r 1`) and quick approvals (`gn ok 1`).
- **High-Definition Terminal Demo**: Regenerated `assets/demo.gif` with frame-quantized clean palettes and sub-50ms execution showcase.

### Changed
- Refactored author metadata to unified email `i.s41m0011@gmail.com`.
- Updated VS Code extension categories to `SCM Providers` and `Collaboration`.

---

## [0.1.5] - 2026-09-23

### Added
- **`git-notes doctor`**: Health-check command verifying Git repo validity, fetch refspecs, auto-sync hooks, and stale notes referencing deleted files.
- **`git-notes blame`**: Git blame with inline note injections anchored to authors and commits.
- **`git-notes import-pr`**: Pull request comment importer that converts GitHub PR reviews into local Git notes.
- **`git-notes summarize`**: AI-powered thread summarizer via Google Gemini.
- **`git-notes init`**: One-second initialization command configuring fetch refspecs and installing git hooks.

---

## [0.1.0] - 2026-09-22

### Added
- **Initial release of the `git-notes` monorepo**:
  - `gn-core`: Git notes engine powered by `gix` and Git plumbing commands.
  - `gn-sync`: Decentralized sync layer with union and Last-Write-Wins (LWW) conflict-free merge strategies.
  - `gn-cli`: Full command-line interface (`add`, `list`, `show`, `sync`, `resolve`, `export`).
  - `gn-tui`: Three-pane interactive terminal browser powered by `ratatui` and `syntect`.
  - `vscode-git-notes`: VS Code & Cursor extension with gutter annotations, inlay hints, and webview discussion panel.
  - `git-notes-chrome`: Chrome MV3 extension displaying inline notes on GitHub PR diffs.
  - `git-notes-hooks`: Python package for zero-config Git hook installation.
  - `github-bridge`: Go HTTP daemon bridging Git notes with GitHub API.
