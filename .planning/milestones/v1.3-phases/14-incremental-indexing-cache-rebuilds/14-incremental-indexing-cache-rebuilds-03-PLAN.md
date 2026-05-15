---
status: complete
---

# Plan 14.3: Freshness Diagnostics And Status Counts

## Goal

Make storage freshness observable through typed status reports and CLI-readable compatibility information.

## Tasks

- Add `StorageStatusReport`.
- Count file, symbol, pack, benchmark, and proof rows.
- Surface compatibility and diagnostics.

## Verification

- `ctxpack storage status --repo <repo>`

