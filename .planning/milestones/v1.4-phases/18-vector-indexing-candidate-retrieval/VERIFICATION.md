# Phase 18 Verification

- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-index -p ctxpack-compiler -p ctxpack --test cli_compat`
- `CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-semantic.sh`
