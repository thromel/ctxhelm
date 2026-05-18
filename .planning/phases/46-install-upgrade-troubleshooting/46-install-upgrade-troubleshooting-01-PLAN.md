# Phase 46 Plan: Install, Upgrade & Troubleshooting

**Created:** 2026-05-18
**Status:** Complete

## Scope

Implement INSTALL-01 through INSTALL-04 by adding read-only install diagnostics
and connecting them through release, quickstart, agent setup, and
troubleshooting docs.

## Steps

1. Inspect existing install docs, setup-check behavior, release docs, and CLI
   compatibility tests.
2. Add `ctxpack doctor` with JSON and Markdown output.
3. Verify active binary path, `--version`, `--help`, optional release manifest,
   and optional repo-local state compatibility.
4. Update README, quickstart, release guide, agent setup matrix, and
   troubleshooting guide.
5. Extend docs consistency checks.
6. Add CLI compatibility tests.
7. Run formatting, docs checks, CLI tests, release packaging tests, and CLI help.

## Verification

- `cargo fmt --all --check`
- `bash scripts/check-release-docs.sh`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase46-cli cargo test -p ctxpack --test cli_compat -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-release-packaging cargo test -p ctxpack --test release_packaging -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase46-cli cargo run -p ctxpack -- --help`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase46-cli cargo run -p ctxpack -- doctor --help`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target-phase46-cli cargo run -p ctxpack -- doctor --repo . --binary /tmp/ctxpack-target-phase46-cli/debug/ctxpack --release-manifest /tmp/ctxpack-phase45-dist/ctxpack-v1.1.0-aarch64-apple-darwin.manifest.json --format json`
