# Deferred Items

## 2026-05-13 - Broken duplicate git ref

- **Found during:** Plan 02 summary self-check
- **Observation:** `git log --oneline --all` failed with `fatal: bad object refs/heads/master 2`.
- **Impact:** Direct commit-object checks with `git cat-file -e <hash>^{commit}` passed, so Plan 02 commit verification was not blocked.
- **Deferred because:** The broken duplicate ref is a repository hygiene issue outside the retrieval-ranking plan scope.
