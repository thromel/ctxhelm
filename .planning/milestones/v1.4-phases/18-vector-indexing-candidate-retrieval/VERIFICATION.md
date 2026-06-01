# Phase 18 Verification

- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-index -p ctxhelm-compiler -p ctxhelm --test cli_compat`
- `CTXHELM_BIN=/tmp/ctxhelm-target/debug/ctxhelm bash scripts/smoke-semantic.sh`
