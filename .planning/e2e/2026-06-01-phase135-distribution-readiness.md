# Phase 135 Distribution Readiness

## Goal

Prove the next install-channel step without publishing package-manager artifacts.

Phase 135 keeps the supported public path as the GitHub archive release, while
making the deferred Homebrew and crates.io channels mechanically checkable.

## Changes

- Added `scripts/render-homebrew-formula.sh` to render the Homebrew formula
  template from a concrete version, archive URL, and SHA-256 digest.
- Extended `scripts/smoke-distribution-metadata.sh` to validate renderer syntax,
  render the formula from the exact packaged archive digest when
  `CTXHELM_DIST_DIR` is available, and check the crates package file boundary.
- Passed `CTXHELM_DIST_DIR` from `scripts/release-gate.sh` into the distribution
  metadata smoke so the formula digest matches the archive produced by the gate.
- Updated release/distribution/crates docs and release-doc drift checks.
- Added release-packaging tests for the distribution readiness script contract.

## Proof

Durable source-free proof:

- `.ctxhelm/e2e/phase135-distribution-readiness.json`

Key fields:

- `schemaVersion = ctxhelm-distribution-readiness-v1`
- `homebrewFormulaRender.status = passed`
- `homebrewFormulaRender.archiveName = ctxhelm-v1.1.3-aarch64-apple-darwin.tar.gz`
- `cratesPackage.status = passed`
- `cratesPackage.sourceFreeBoundaryChecked = true`
- `privacyStatus.localOnly = true`
- `unsupportedActions` still includes `brew tap publication`, `crates.io publish`,
  `global install`, `signed installer`, and `self-update`

## Validation

Commands run from `/Users/romel/Documents/GitHub/ctxhelm-release-gate-clean-20260601`:

```bash
CTXHELM_DIST_DIR="$PWD/dist" bash scripts/smoke-distribution-metadata.sh
bash scripts/check-release-docs.sh
bash -n scripts/smoke-distribution-metadata.sh
bash -n scripts/release-gate.sh
bash -n scripts/render-homebrew-formula.sh
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/ctxhelm-phase135-target cargo test -p ctxhelm --test release_packaging --locked -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-phase135-target cargo test --workspace --locked
CARGO_TARGET_DIR=/tmp/ctxhelm-phase135-target cargo run -p ctxhelm --locked -- --help
git diff --check
```

All passed.

## Non-Goals

No Homebrew tap publication, crates.io publication, global install mutation,
signed installer, cloud service, or self-update feature was added.
