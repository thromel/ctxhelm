---
status: complete
---

# Plan 14.1: Storage-Backed File Fingerprinting

## Goal

Persist safe inventory file records into SQLite and classify each re-indexed path as reused, created, updated, or deleted.

## Tasks

- Add typed storage sync report contracts.
- Compare current safe inventory against stored file rows.
- Upsert changed records and delete missing records.
- Keep JSON inventory fallback untouched.

## Verification

- `cargo test -p ctxhelm-index storage`

