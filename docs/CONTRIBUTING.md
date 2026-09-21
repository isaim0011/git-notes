# Contributing to git-notes

## Development Setup
- Rust: `cargo build`
- TypeScript: `bun install`
- Python: `pip install -e ".[dev]"`
- Go: `go build ./...`

## Running Tests
- Rust: `cargo test`
- TypeScript: `bun test`
- Python: `pytest`
- Go: `go test ./...`

## Coding Style
- Rust: run `cargo fmt` and `cargo clippy`.
- TypeScript: use ESLint/Prettier as configured.
- Python: use `ruff`.
- Go: use `gofmt` and `staticcheck`.

## Pull Request Process
1. Fork the repo and create a branch.
2. Ensure all tests pass.
3. Submit a PR with a clear description of the changes.

## Issue Labels
- `bug`: Something is broken.
- `enhancement`: New feature or request.
- `good first issue`: Good for newcomers.
