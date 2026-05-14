---
status: complete
---

# Plan 14.4: Source-Free Incremental Tests

## Goal

Prove incremental storage sync works and does not persist source bodies.

## Tasks

- Test create/reuse/update/delete count transitions.
- Test status row counts after sync.
- Test source sentinel absence from the SQLite bytes.

## Verification

- `cargo test -p ctxpack-index storage`

