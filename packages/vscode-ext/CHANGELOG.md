# Changelog

All notable changes to the `git-notes` VS Code / Cursor / Windsurf extension are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.9] - 2026-09-24

### Added
- **Interactive Review & Star Triggers**: Added non-intrusive feedback prompt after 3 successful user interactions to easily review on Open VSX or star the repository on GitHub.
- **In-Panel Feedback & Star Buttons**: Added `💬 Give Feedback` and `⭐ Star Repo` directly inside the thread webview.
- **Rebase-Heal Support**: Support for automatically maintaining gutter decorations and discussion threads across commit rebases and amends.
- **Offline P2P & Multi-Provider Sync**: Full integration with the latest `gn-cli` v0.1.8 capabilities.

---

## [0.1.7] - 2026-09-24

### Added
- **Default Keyboard Shortcuts**:
  - `Alt+N` (`Cmd+Option+N` on macOS): Add note to focused editor line.
  - `Alt+Shift+N` (`Cmd+Option+Shift+N` on macOS): Open note discussion thread panel.
  - `Alt+S` (`Cmd+Option+S` on macOS): Instant note sync with remote `refs/notes/*`.
- **Live Dynamic Shields**: Updated all badges to live dynamic SVG shields.

---

## [0.1.6] - 2026-09-24

### Added
- **Adaptive User Learning Integration**: Real-time integration with `gn-cli`'s behavior profile engine.
- **Metadata Update**: Unified publisher and author metadata to `Bimo <i.s41m0011@gmail.com>`.
- **Extension Categories**: Set official marketplace categories to `SCM Providers` and `Collaboration`.

---

## [0.1.0] - 2026-09-22

### Added
- Initial release of the `git-notes` VS Code and Cursor extension.
- Inline editor gutter annotations for code comments.
- Inlay hints showing comment authors and line counts.
- Interactive Webview discussion panel with threaded replies, approvals, and resolutions.
