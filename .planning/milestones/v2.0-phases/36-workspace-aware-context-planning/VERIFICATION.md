# Phase 36 Verification

Verified in this development pass:

```bash
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-core workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-compiler workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm workspace --test cli_compat -- --nocapture
cargo fmt --all --check
bash scripts/check-release-docs.sh
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo run -p ctxhelm -- workspace get-pack --help
CTXHELM_BIN=/tmp/ctxhelm-target/debug/ctxhelm bash scripts/smoke-workspace.sh
```

Status: passed.

