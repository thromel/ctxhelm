# Phase 17 Verification

- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-index -p ctxhelm-compiler`
- `CTXHELM_BIN=/tmp/ctxhelm-target/debug/ctxhelm bash scripts/smoke-semantic.sh`
