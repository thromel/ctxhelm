# Phase 17 Verification

- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-index -p ctxpack-compiler`
- `CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-semantic.sh`
