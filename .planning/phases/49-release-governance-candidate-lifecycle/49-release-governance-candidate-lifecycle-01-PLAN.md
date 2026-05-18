# Phase 49 Plan: Release Governance & Candidate Lifecycle

## Tasks

1. Add source-free candidate status creation and validation for ready,
   deferred, and blocked lifecycle states.
2. Add safe rollback for marked candidate artifact directories, with optional
   previous-metadata restore.
3. Document deterministic protocol proof, optional real-client proof, candidate
   statuses, and rollback boundaries.
4. Add a release checklist covering required gates, optional proof, known
   limitations, and rollback.
5. Wire governance smoke into docs checks, release gate, and release packaging
   contract tests.

## Verification

- `bash scripts/smoke-release-governance.sh`
- `bash scripts/check-release-docs.sh`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase49 cargo test -p ctxpack --test release_packaging -- --nocapture`

