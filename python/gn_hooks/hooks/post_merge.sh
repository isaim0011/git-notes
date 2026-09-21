#!/usr/bin/env bash
# git-notes hook: auto-fetch notes after merge
set -euo pipefail
git-notes sync pull --quiet 2>/dev/null || true
