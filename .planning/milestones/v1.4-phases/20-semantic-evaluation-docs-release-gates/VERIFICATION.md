# Phase 20 Verification

- `bash scripts/check-release-docs.sh`
- `CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-semantic.sh`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo run -p ctxpack -- --help`
