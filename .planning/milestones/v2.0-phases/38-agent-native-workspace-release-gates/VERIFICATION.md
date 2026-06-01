# Phase 38 Verification

Verified in this development pass:

```bash
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-mcp resources -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm script_contract --test release_packaging -- --nocapture
cargo fmt --all --check
bash scripts/check-release-docs.sh
CTXHELM_BIN=/tmp/ctxhelm-target/debug/ctxhelm bash scripts/smoke-shared-artifacts.sh
```

Status: passed.

