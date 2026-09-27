# Sync, CI Gating & Rebase Healing

Keep your team notes in sync across git remotes, verify code quality in CI, and survive git rebases effortlessly.

### Instant Sync & Healing
- Press `Alt+S` (`Cmd+Option+S` on macOS) or trigger **git-notes: Sync Notes** to push/pull notes with your remote.
- Run `gn heal` in your terminal whenever you rebase or amend commits to preserve note anchors.
- Use `gn check --no-unresolved` in CI to prevent merging until all review discussions are resolved!

[Sync Notes with Remote](command:git-notes.sync)
