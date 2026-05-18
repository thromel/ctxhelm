# Phase 37 Verification

Verified in this development pass:

```bash
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-core shared -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-index shared -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack workspace --test cli_compat -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack workspace_shared_artifacts_and_team_policy_are_source_free --test cli_compat -- --nocapture
cargo fmt --all --check
bash scripts/check-release-docs.sh
CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-shared-artifacts.sh
```

Status: passed.

