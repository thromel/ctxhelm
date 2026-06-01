---
status: complete
---

# Plan 14.2: CLI Incremental Sync Surface

## Goal

Expose storage-backed incremental indexing through the existing CLI without making SQLite mandatory for all users.

## Tasks

- Add `ctxhelm index --store`.
- Add optional `--store-path`.
- Print source-free storage sync counts.
- Preserve the existing inventory report.

## Verification

- `CTXHELM_HOME=<tmp> cargo run -q -p ctxhelm -- index --repo <repo> --store`

