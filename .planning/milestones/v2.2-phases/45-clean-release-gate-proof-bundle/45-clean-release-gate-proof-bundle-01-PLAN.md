# Phase 45 Plan: Clean Release Gate & Proof Bundle

**Created:** 2026-05-18
**Status:** Complete

## Scope

Implement REL-01 through REL-04 by hardening the existing release scripts,
release docs, and release packaging tests.

## Steps

1. Inspect existing `scripts/release-gate.sh`, `scripts/release-package.sh`,
   `scripts/audit-release-artifact.sh`, `docs/release.md`, and release
   packaging tests.
2. Add machine-readable artifact audit reports.
3. Add package release manifests with archive and binary SHA-256 identity.
4. Add source-free release proof bundle output from the release gate.
5. Make package builds honor `CARGO_TARGET_DIR`.
6. Update release docs and docs consistency checks.
7. Extend focused release packaging tests.
8. Run focused tests, actual packaging, and the full deterministic release gate.

## Verification

- `bash -n scripts/audit-release-artifact.sh`
- `bash -n scripts/release-package.sh`
- `bash -n scripts/release-gate.sh`
- `bash -n scripts/check-release-docs.sh`
- `bash scripts/check-release-docs.sh`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-release-packaging cargo test -p ctxpack --test release_packaging -- --nocapture`
- `CTXPACK_ALLOW_DIRTY=1 CTXPACK_DIST_DIR=/tmp/ctxpack-phase45-dist CARGO_TARGET_DIR=/tmp/ctxpack-target-phase45-release bash scripts/release-package.sh`
- `CTXPACK_ALLOW_DIRTY=1 CTXPACK_SKIP_REAL_CLIENT=1 CTXPACK_DIST_DIR=/tmp/ctxpack-phase45-gate-dist CTXPACK_PROOF_DIR=/tmp/ctxpack-phase45-proof CARGO_TARGET_DIR=/tmp/ctxpack-target-phase45-gate bash scripts/release-gate.sh`
