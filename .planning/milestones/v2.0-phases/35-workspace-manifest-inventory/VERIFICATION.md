# Phase 35 Verification

Verified on 2026-05-17.

## Commands

```bash
cargo fmt --all --check
bash scripts/check-release-docs.sh
CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-workspace.sh
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo run -p ctxpack -- --help
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-core workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-index workspace -- --test-threads=1 --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack workspace --test cli_compat -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test --workspace
```

## Result

All commands passed.

## Evidence

- `ctxpack --help` lists the `workspace` command.
- Core workspace contract test passed.
- Index workspace manifest/status tests passed.
- CLI compatibility workspace init/status test passed with two temp git repos.
- Workspace smoke passed and verified the source sentinel did not appear in output or local ctxpack state.
- Full workspace test suite passed.
