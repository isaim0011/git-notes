# gn-core

> **The core engine for `git-notes` — reading, writing, and merging decentralized Git notes with zero commit rewriting.**

[![Crates.io](https://img.shields.io/crates/v/gn-core.svg)](https://crates.io/crates/gn-core)
[![Docs.rs](https://docs.rs/gn-core/badge.svg)](https://docs.rs/gn-core)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/isaim0011/git-notes)

`gn-core` provides high-performance Git plumbing abstractions for creating, querying, and updating notes in custom references like `refs/notes/comments`, `refs/notes/review`, and `refs/notes/todos`.

---

## ⚡ Features

- **Zero History Modification**: Operates strictly on Git objects (`hash-object`, `mktree`, `commit-tree`, `update-ref`).
- **Conflict-Free Merging**: Built-in `UnionStrategy` and `LwwStrategy` (Last-Write-Wins) for decentralized sync.
- **Threaded Discussions**: Full support for hierarchical note threads anchored to specific files and line numbers.
- **Pure Rust**: Safe, fast, and cross-platform.

---

## 🚀 Quick Usage

Add `gn-core` to your `Cargo.toml`:

```toml
[dependencies]
gn-core = "0.1"
```

### Writing a Note

```rust
use gn_core::{Note, NotesEngine, Namespace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = NotesEngine::new(".");
    
    let note = Note::new(
        "36303fd".to_string(),               // commit SHA
        Some("src/main.rs".to_string()),    // file path
        Some(42),                           // start line
        Some(45),                           // end line
        "Consider caching this query".into(), // markdown body
        "Alice <alice@example.com>".into(),  // author
        Namespace::Comments,                // namespace
    );

    let commit_hash = engine.write_note(&note)?;
    println!("Updated notes ref in commit: {}", commit_hash);

    Ok(())
}
```

### Reading Notes

```rust
use gn_core::{NotesEngine, Namespace};

let engine = NotesEngine::new(".");
let notes = engine.read_notes(&Namespace::Comments)?;

for note in notes {
    println!("{}:{} -> {}", note.file.unwrap_or_default(), note.line_start.unwrap_or(0), note.body);
}
```

---

## 🔗 Repository

Part of the **[git-notes](https://github.com/isaim0011/git-notes)** project.
