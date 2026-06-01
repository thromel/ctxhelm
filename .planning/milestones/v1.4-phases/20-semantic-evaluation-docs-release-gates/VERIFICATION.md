# Phase 20 Verification

- `bash scripts/check-release-docs.sh`
- `CTXHELM_BIN=/tmp/ctxhelm-target/debug/ctxhelm bash scripts/smoke-semantic.sh`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo run -p ctxhelm -- --help`
