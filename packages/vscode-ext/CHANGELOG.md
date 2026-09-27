# Changelog

All notable changes to the `git-notes` ecosystem and the VS Code / Cursor extension are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.13] - 2026-09-27

### Added
- **Asynchronous Decoupled Task Queue (`AsyncRunner`)**: All git operations and CLI subprocess calls are now processed asynchronously with queued scheduling and debouncing, ensuring 100% fluid editor performance with zero UI thread freezes during high-frequency saving or rapid git activity.
- **Automated CLI Binary Installer**: Automatically detects missing `git-notes` / `gn` binaries on the user's host system and securely downloads the verified native binary for Windows, macOS (Apple Silicon & Intel), or Linux in 1 click without requiring Cargo or Rust toolchains.
- **Interactive Onboarding Walkthrough**: Rich built-in VS Code walkthrough (`contributes.walkthroughs`) guiding new developers step-by-step through repository initialization, inline gutter notes (`Alt+N`), webview discussion threads (`Alt+Shift+N`), and rebase-healing.
- **Data & Refspec Integrity Validator (`gn validate` / `gn fsck`)**: Audits all `refs/notes/*`, validates JSON schemas and Git commit DAG anchors, detecting and preventing any reference drift or corruption.
- **Modernized Interactive TUI (`git-notes-tui`)**:
  - Live cursor tracking with bright `▶` indicators and background line highlights in the code diff view.
  - Expanded Review & Notes panel to 40% width with clean word-wrapping to prevent border clipping.
  - Keyboard navigation synced between file tree, diff, and note threads.

---

## [0.1.11] - 2026-09-27

### Added
- **Data & Refspec Integrity Validator (`gn validate` / `gn fsck` / `gn lint`)**: Full tree and content-addressed blob validation across all `refs/notes/*`. Checks commit DAG anchors, detects orphan notes or corruption, and supports `--strict` CI gating and `--json` structured diagnostics.
- **13-Stage Burden & Stress Test Suite**: Expanded `burden_test.py` with Test 13 ensuring 100% data integrity validation.

---

## [0.1.10] - 2026-09-27

### Added
- **CI Quality Gating & Merge-Readiness (`gn check` / `gn gate`)**: Single-command automated merge validation with `--min-approvals <N>` and `--no-unresolved`.
- **Cryptographic Note Verification (`gn a --sign` / `gn verify`)**: GPG & SSH signature support for authentic, tamper-proof reviews in Git.
- **Shell Auto-Completions with 1-Click Install (`gn completions --install`)**: Zero-config auto-detection and installation for Bash, Zsh, Fish, PowerShell, and Elvish.
- **12-Stage Real-World Multi-Node Burden Test Suite (`burden_test.py`)**: End-to-end multi-developer simulation enforcing zero regressions across all core features.

### Fixed
- **Clean Machine JSON Output**: Suppressed plain-text interactive workflow suggestions when `--json` flags are passed (`gn list --json`, `gn check --json`), ensuring 100% compliant JSON parsing for CI/CD runners and external tooling.

---

## [0.1.9] - 2026-09-26

### Added
- **CI Quality Gating & Merge-Readiness (`gn check` / `gn gate`)**: Single-command automated gating for CI/CD pipelines (`--min-approvals <N>`, `--no-unresolved`, `--namespace`, `--commit`).
- **Cryptographic Note Signing & Verification (`gn a --sign` / `gn verify`)**: GPG & SSH signature creation and verification.
- **Shell Auto-Completions with 1-Click Install (`gn completions --install`)**: Zero-config shell completions.

---

## [0.1.8] - 2026-09-24

### Added
- **`gn rebase-heal` (Semantic Commit Re-anchoring)**: Automatically detects when branch commits have been rewritten or rebased (`git rebase`, `commit --amend`, `cherry-pick`) and re-anchors orphaned notes to new commit SHAs using stable patch-IDs and commit subject heuristics. Supports `--dry-run`.
- **Multi-Provider Importers (`gn import` / `gn imp`)**: GitLab MR, Bitbucket PR, and Jira issue importers.
- **Offline Team P2P & USB Sync (`gn sync`)**: P2P Wi-Fi daemon and portable `.bundle` packages for offline transfers.
- **Upgraded Interactive TUI (`git-notes-tui`)**: Real syntax-highlighted diffs and Markdown thread cards.
- **Gentle Community Engagement**: In-editor star and review prompts.

---

## [0.1.0] - 2026-09-22

### Added
- Initial release of the `git-notes` extension for Visual Studio Code, Cursor, and Windsurf IDEs.
