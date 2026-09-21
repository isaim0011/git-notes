# Architecture

## Notes Storage
git-notes leverages the native Git object database to store comments and metadata. By using commands like `hash-object`, `mktree`, and `commit-tree`, it builds a parallel history (usually under `refs/notes/*`) without cluttering the main project history.

## Namespaces
We use namespaces (e.g. `refs/notes/comments`, `refs/notes/reviews`) to isolate different types of metadata. This allows clients to fetch only the data they care about.

## Merge Strategies
To handle decentralized collaboration, git-notes employs a Union merge strategy combined with Last-Writer-Wins (LWW) semantics for conflict resolution. This ensures that concurrent edits don't result in unmergeable branches.

## Sidecar Model
For IDE extensions (VS Code, IntelliJ), git-notes runs as a sidecar process. The extensions communicate with the core Rust binary via JSON-RPC over stdio, keeping the core logic centralized.

## GitHub Bridge
The GitHub Bridge is a service that synchronizes git-notes with GitHub Pull Requests and Issues. It listens to webhooks and translates GitHub comments into git-notes objects, and vice versa.

## Extension Points
New surfaces and namespaces can be added by implementing the relevant traits in the core Rust library and registering them in the CLI dispatcher.
