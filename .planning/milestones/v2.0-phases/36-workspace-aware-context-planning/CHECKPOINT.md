# Phase 36 Checkpoint: Workspace-Aware Context Planning

## Status

Started on 2026-05-17.

## Completed In This Checkpoint

- Added `WorkspaceContextPlan` and `WorkspaceRepoPlan` contracts.
- Added compiler support for workspace-aware `prepare-task`.
- Added CLI support:
  - `ctxpack workspace prepare-task <task> --repo <path>`
- Added source-free routing tests:
  - core contract test
  - compiler repo-routing test
  - CLI compatibility test
  - workspace smoke extension
- Added component-level architecture docs.

## Still Pending For Full Phase 36 Completion

- Workspace-aware pack output.
- Budget warnings for workspace plans/packs.
- Stronger validation guidance across selected repositories.
- Formal Phase 36 plan and completion summary.
- Any MCP/resource exposure, if kept in Phase 36 rather than Phase 38.

## Verification

All passed:

```bash
cargo fmt --all --check
bash scripts/check-release-docs.sh
CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-workspace.sh
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo run -p ctxpack -- workspace prepare-task --help
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-core workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-compiler workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack workspace --test cli_compat -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test --workspace
```
