# Phase 35 Verification

Verified on 2026-05-17.

## Commands

```bash
cargo fmt --all --check
bash scripts/check-release-docs.sh
CTXHELM_BIN=/tmp/ctxhelm-target/debug/ctxhelm bash scripts/smoke-workspace.sh
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo run -p ctxhelm -- --help
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-core workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-index workspace -- --test-threads=1 --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm workspace --test cli_compat -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test --workspace
```

## Result

All commands passed.

## Evidence

- `ctxhelm --help` lists the `workspace` command.
- Core workspace contract test passed.
- Index workspace manifest/status tests passed.
- CLI compatibility workspace init/status test passed with two temp git repos.
- Workspace smoke passed and verified the source sentinel did not appear in output or local ctxhelm state.
- Full workspace test suite passed.
