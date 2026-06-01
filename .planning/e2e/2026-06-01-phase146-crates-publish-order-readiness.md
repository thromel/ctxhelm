# Phase 146: Crates Publish-Order Readiness

## Goal

Make the crates.io source-distribution path mechanically honest: every workspace
crate must have a checked package boundary, internal path dependencies must carry
publishable versions, and docs must state the required publish order without
claiming crates.io publication.

## Issue Found

`cargo package --manifest-path crates/ctxhelm/Cargo.toml --locked --no-verify`
failed because internal path dependencies did not specify versions:

```text
error: all dependencies must have a version specified when packaging.
dependency `ctxhelm-compiler` does not specify a version
```

After adding explicit internal dependency versions, a top-level dry-run still
cannot complete until the internal crates are actually published to crates.io.
That is the correct crates.io constraint: dependent crates resolve internal
dependencies from the registry during verification.

## Changes

- Added explicit `version = "1.1.9"` metadata to internal path dependencies
  across `ctxhelm-index`, `ctxhelm-compiler`, `ctxhelm-mcp`, and `ctxhelm`.
- Updated `scripts/smoke-distribution-metadata.sh` to run
  `cargo package --manifest-path ... --list` for all workspace crates.
- Kept the full non-publishing package dry-run scoped to the leaf
  `ctxhelm-core` crate, where it can succeed before any internal crate is
  published.
- Recorded the required crates.io publish order:
  1. `ctxhelm-core`
  2. `ctxhelm-index`
  3. `ctxhelm-compiler`
  4. `ctxhelm-mcp`
  5. `ctxhelm`
- Updated release docs and release-doc checks to describe the source
  distribution boundary accurately.
- Removed old candidate-name references from the brand guide so public naming is
  consistently `ctxhelm`.

## Proof Artifacts

- `.ctxhelm/distribution-metadata-smoke.json`
  - `cratesPackage.status = "publish_order_ready"`
  - `packageListCheckedCrates = ["ctxhelm-core", "ctxhelm-index", "ctxhelm-compiler", "ctxhelm-mcp", "ctxhelm"]`
  - `leafDryRunCheckedCrates = ["ctxhelm-core"]`
  - `dependentDryRunStatus = "blocked_until_internal_crates_are_published_in_order"`
  - `publishOrder = ["ctxhelm-core", "ctxhelm-index", "ctxhelm-compiler", "ctxhelm-mcp", "ctxhelm"]`

## Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `bash scripts/smoke-distribution-metadata.sh`
- `bash scripts/check-release-docs.sh`
- `cargo test --workspace --locked --no-fail-fast`
- `CTXHELM_ALLOW_DIRTY=1 bash scripts/release-gate.sh`
- `rg -n -i "ctxpack|repo context packer|needlepath|winnow|contextmason|repolens|thromel/ctxpack|repo packer" -S . --glob '!target/**' --glob '!dist/**' --glob '!vendor/**' --glob '!Cargo.lock'`
- Portfolio/CV old-name scan in `/Users/romel/Documents/GitHub/thromel.github.io` with the same query, excluding generated site and node modules.

## Public State

- GitHub repository name and remote: `thromel/ctxhelm`.
- Latest pre-change public CI run for `2a23138b6605ea8403e74f90ff3b663688dde260`
  passed.
- Portfolio project page and CV reference `ctxhelm` and
  `https://github.com/thromel/ctxhelm`.
- crates.io publication remains deferred. The release metadata now explains the
  correct publish order instead of presenting an impossible top-level dry-run as
  an immediate readiness check.
