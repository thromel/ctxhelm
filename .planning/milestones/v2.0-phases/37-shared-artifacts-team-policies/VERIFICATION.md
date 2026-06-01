# Phase 37 Verification

Verified in this development pass:

```bash
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-core shared -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-index shared -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm workspace --test cli_compat -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm workspace_shared_artifacts_and_team_policy_are_source_free --test cli_compat -- --nocapture
cargo fmt --all --check
bash scripts/check-release-docs.sh
CTXHELM_BIN=/tmp/ctxhelm-target/debug/ctxhelm bash scripts/smoke-shared-artifacts.sh
```

Status: passed.

