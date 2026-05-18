# Phase 36 Verification

Verified in this development pass:

```bash
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-core workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-compiler workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack workspace --test cli_compat -- --nocapture
cargo fmt --all --check
bash scripts/check-release-docs.sh
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo run -p ctxpack -- workspace get-pack --help
CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-workspace.sh
```

Status: passed.

