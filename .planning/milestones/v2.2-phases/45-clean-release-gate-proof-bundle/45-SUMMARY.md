# Phase 45 Summary: Clean Release Gate & Proof Bundle

**Completed:** 2026-05-18
**Status:** Complete

## Delivered

- Added package-time release manifests next to archives.
- Added machine-readable artifact audit reports with source-free privacy status.
- Extended `sha256sums.txt` to cover archive, manifest, and audit report.
- Added release-gate proof bundle summaries with binary identity, archive
  identity, required check outcomes, optional benchmark/client proof status, and
  privacy status.
- Made `scripts/release-package.sh` honor `CARGO_TARGET_DIR` so maintainers can
  use clean temporary target directories during release verification.
- Updated release docs and docs consistency checks for manifest, audit report,
  proof bundle, and `CARGO_TARGET_DIR`.
- Extended focused release packaging tests for the new metadata and proof-bundle
  contracts.

## Validation

- `bash -n scripts/audit-release-artifact.sh && bash -n scripts/release-package.sh && bash -n scripts/release-gate.sh && bash -n scripts/check-release-docs.sh`
- `bash scripts/check-release-docs.sh`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-release-packaging cargo test -p ctxpack --test release_packaging -- --nocapture`
- `CTXPACK_ALLOW_DIRTY=1 CTXPACK_DIST_DIR=/tmp/ctxpack-phase45-dist CARGO_TARGET_DIR=/tmp/ctxpack-target-phase45-release bash scripts/release-package.sh`
- `CTXPACK_ALLOW_DIRTY=1 CTXPACK_SKIP_REAL_CLIENT=1 CTXPACK_DIST_DIR=/tmp/ctxpack-phase45-gate-dist CTXPACK_PROOF_DIR=/tmp/ctxpack-phase45-proof CARGO_TARGET_DIR=/tmp/ctxpack-target-phase45-gate bash scripts/release-gate.sh`

## Proof Evidence

- Full deterministic release gate passed.
- Workspace tests passed inside the release gate: 256 Rust tests plus doc tests.
- Release package produced:
  - `ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz`
  - `ctxpack-v1.1.0-aarch64-apple-darwin.manifest.json`
  - `ctxpack-v1.1.0-aarch64-apple-darwin.audit.json`
  - `ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz.sha256`
  - `sha256sums.txt`
- Proof bundle summary was written to
  `/tmp/ctxpack-phase45-proof/release-proof-summary.json`.
- Proof summary recorded `privacyStatus.localOnly = true`,
  `sourceTextLogged = false`, benchmark proof `skipped`, Codex/Claude
  real-client proof `skipped`, and Cursor/OpenCode proof `not_claimed`.

## Notes

- The release gate remains non-publishing: no tags, pushes, release uploads,
  registry publishing, Homebrew tap changes, signing, or global agent config
  mutation.
- The persisted proof bundle uses file names and checksums rather than
  machine-local source paths.
