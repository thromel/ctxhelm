# Phase 49 Summary: Release Governance & Candidate Lifecycle

Status: Complete

## Completed

- Added `scripts/release-candidate-status.sh` for source-free candidate status
  creation and validation.
- Added `scripts/release-candidate-rollback.sh` for safe rollback of marked
  local candidate artifact directories and previous-metadata restore.
- Added `scripts/smoke-release-governance.sh`.
- Added `docs/release-governance.md`.
- Added `packaging/release/release-checklist.md`.
- Wired governance smoke into `docs/release.md`, `scripts/release-gate.sh`,
  `scripts/check-release-docs.sh`, `README.md`, and release packaging contract
  tests.

## Verification

- `bash scripts/smoke-release-governance.sh` passed.
- `bash scripts/check-release-docs.sh` passed.
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase49 cargo test -p ctxhelm --test release_packaging -- --nocapture` passed.

