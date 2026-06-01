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
2. Add `ctxhelm doctor` with JSON and Markdown output.
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
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase46-cli cargo test -p ctxhelm --test cli_compat -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target-release-packaging cargo test -p ctxhelm --test release_packaging -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase46-cli cargo run -p ctxhelm -- --help`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase46-cli cargo run -p ctxhelm -- doctor --help`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase46-cli cargo run -p ctxhelm -- doctor --repo . --binary /tmp/ctxhelm-target-phase46-cli/debug/ctxhelm --release-manifest /tmp/ctxhelm-phase45-dist/ctxhelm-v1.1.0-aarch64-apple-darwin.manifest.json --format json`
