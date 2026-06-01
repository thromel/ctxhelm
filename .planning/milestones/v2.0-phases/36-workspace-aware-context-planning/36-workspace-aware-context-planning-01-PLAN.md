# Phase 36 Plan: Workspace-Aware Context Planning

## Goal

Route tasks across a local workspace before file-level retrieval and compile
repo-boundary-aware workspace context packs.

## Scope

- Extend workspace planning from status-only metadata to `WorkspaceContextPlan`.
- Preserve repo IDs, labels, tags, path labels, reasons, confidence, and
  workspace provenance in plan output.
- Add `WorkspaceContextPack` with per-repo nested `ContextPack` objects under
  `repoPacks`.
- Keep single-repo `prepare-task` and `get-pack` behavior unchanged unless the
  caller explicitly uses `ctxhelm workspace ...`.
- Add source-free tests, docs, and smoke coverage.

## Verification

- `cargo test -p ctxhelm-core workspace`
- `cargo test -p ctxhelm-compiler workspace`
- `cargo test -p ctxhelm workspace --test cli_compat`
- `scripts/smoke-workspace.sh`

