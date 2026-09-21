#!/usr/bin/env bash
# git-notes hook: auto-push notes before push
set -euo pipefail
git-notes sync push --quiet 2>/dev/null || true
