# Welcome to git-notes

**git-notes** provides decentralized, git-native code annotations, PR review discussions, and todos directly anchored to git commits without cluttering your code or git log.

### Step 1: Initialize Your Repository
Configure your repository's git refspec and synchronization hooks in one click:

[Initialize Repository](command:git-notes.sync)

- Configures `remote.origin.fetch` to track `+refs/notes/*:refs/notes/*`.
- Sets up non-intrusive `post-merge` and `pre-push` hooks.
