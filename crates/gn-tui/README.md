# gn-tui (`git-notes-tui`)

> **Interactive terminal user interface for `git-notes` — browse code hunks, inspect inline comments, and reply to review threads.**

[![Crates.io](https://img.shields.io/crates/v/gn-tui.svg)](https://crates.io/crates/gn-tui)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/isaim0011/git-notes)

Part of the **[git-notes](https://github.com/isaim0011/git-notes)** ecosystem.

---

## ⌨️ Keyboard Shortcuts & Navigation

| Keystroke | Action |
|---|---|
| `↑` / `↓` or `k` / `j` | Navigate files and notes list |
| `Enter` or `→` | Expand file / focus diff view & note threads |
| `Esc` or `←` | Return to previous panel |
| `r` | Quick reply to selected note thread |
| `ok` / `a` | Resolve / approve selected note |
| `q` | Quit TUI |

---

## ⚡ Quickies
Inspect notes directly from the command line:
- `gn s` — Arrow-key quick picker
- `gn shortcuts` — Display full quickies & keybindings cheat sheet
- `gn shortcuts --set <alias>=<command>` — Customize bindings
