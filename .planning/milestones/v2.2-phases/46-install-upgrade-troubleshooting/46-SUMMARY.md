# Phase 46 Summary: Install, Upgrade & Troubleshooting

**Completed:** 2026-05-18
**Status:** Complete

## Delivered

- Added `ctxpack doctor` for read-only install and upgrade diagnostics.
- `doctor` verifies:
  - absolute binary path
  - binary exists
  - `ctxpack --version`
  - `ctxpack --help`
  - optional release manifest version/privacy/checksum metadata
  - optional repo-local storage compatibility
- Added JSON and Markdown doctor output.
- Connected `doctor` through README, quickstart, release guide, agent setup
  matrix, and troubleshooting guide.
- Added stale-binary/upgrade-mismatch troubleshooting.
- Extended docs consistency checks and CLI compatibility tests.

## Validation

- `cargo fmt --all --check`
- `bash scripts/check-release-docs.sh`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase46-cli cargo test -p ctxpack --test cli_compat -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-release-packaging cargo test -p ctxpack --test release_packaging -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase46-cli cargo run -p ctxpack -- --help`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase46-cli cargo run -p ctxpack -- doctor --help`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase46-cli cargo run -p ctxpack -- doctor --repo . --binary /tmp/ctxpack-target-phase46-cli/debug/ctxpack --release-manifest /tmp/ctxpack-phase45-dist/ctxpack-v1.1.0-aarch64-apple-darwin.manifest.json --format json`

## Notes

- `setup-check` remains artifact-focused; `doctor` handles install/upgrade
  diagnostics.
- Missing local state is accepted as compatible because a fresh install may not
  have initialized storage yet.
- No install or troubleshooting path mutates global agent configuration.
