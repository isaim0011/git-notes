#!/usr/bin/env bash
# git-notes hook: re-anchor notes to new commit if HEAD changed
set -euo pipefail
git-notes sync auto --quiet 2>/dev/null || true
