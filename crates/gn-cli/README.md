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

### Quickies & Single-Letter Abbreviations (`gn`)
Save keystrokes with fast, ergonomic quickies:
- `gn a -f file.rs -l 42 -m "..."` — Quickie add note
- `gn l` — List notes with compact numbered index `[1]`, `[2]`
- `gn r 1 -m "Fixed!"` — Reply directly using quick index number
- `gn ok 1` — Resolve / approve note by number
- `gn s` — Interactive terminal arrow-key thread picker
- `gn d` — View git diff with inline notes
- `gn b -f file.rs` — Blame with inline notes annotations
- `gn doc` — Doctor health check & stale-note detector
- `gn heal` — Auto-heal & re-anchor notes after git rebase / amend
- `gn check` / `gn gate` — CI quality gating and merge-readiness check
- `gn verify` / `gn sig` — Verify cryptographic signatures of notes
- `gn sum` — AI summary of open discussion threads (Gemini)
- `gn i` — 1-second repo setup (tracking refspec + hooks)
- `gn p2p` — Spin up local Wi-Fi P2P sync server
- `gn imp <gitlab|bitbucket|jira|pr>` — Import reviews from any platform

### Cryptographic Note Signing & Verification (`gn add -s`, `gn verify`)
Cryptographically sign notes with GPG or SSH keys for provenance and tamper detection:
```bash
# Sign a new note with default or configured signing key
gn add -f src/main.rs -l 42 -m "LGTM, verified security constraints" --sign

# Cryptographically sign a reply
gn reply 1 -m "Approved fixes" -s

# Verify signatures across all notes with color badges ([✔ Signed], [⚠ Unsigned], [✗ Bad Signature])
gn verify --all

# Verify a specific note thread or note index
gn verify 1
```

### CI Quality Gating & Readiness (`gn check`, `gn gate`)
Ensure notes and code reviews are satisfied before merging:
```bash
# Verify at least 1 Approved review note and zero unresolved threads
gn check --min-approvals 1 --no-unresolved

# Or using alias:
gn gate --json
```

### Shell Auto-Completions (`gn completions`)
Generate or automatically install tab completions for `bash`, `zsh`, `fish`, `powershell`, or `elvish`:
```bash
# Automatically detect shell and install completions into profile/completions directory
gn completions --install

# Or generate completion script directly:
gn completions zsh > ~/.zsh/completions/_gn
gn completions bash > ~/.bash_completion.d/gn
gn completions fish > ~/.config/fish/completions/gn.fish
gn completions powershell >> $PROFILE
```

### View & Customize Shortcuts
```bash
# View all abbreviations and keybindings
gn shortcuts

# Customize your own aliases
gn shortcuts --set c="add -m"

# Reset to defaults
gn shortcuts --reset
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
- **Changelog**: [https://github.com/isaim0011/git-notes/blob/main/CHANGELOG.md](https://github.com/isaim0011/git-notes/blob/main/CHANGELOG.md)
