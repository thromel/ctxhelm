# Deferred Items

## 2026-05-13 - Invalid duplicate local git refs

- **Found during:** Plan 03 self-check
- **Issue:** `git log --oneline --all` fails because local refs named `.git/refs/heads/master 2` and `.git/refs/heads/master 3` are invalid refnames.
- **Impact:** This blocks `git log --all` checks, but does not block normal commits or direct commit object verification with `git cat-file -e`.
- **Deferred because:** The invalid refs are pre-existing local git metadata outside the ctxpack-index module split.
