# Milestone 21: Current Diff Anchors

## Goal

Let agents explicitly use the current safe local diff as active context for `prepare_task` and `get_pack`.

## Scope

- Factor safe current-diff collection into `ctxpack-index`.
- Preserve source-free output and safe inventory filtering for changed paths.
- Add `includeCurrentDiff` to MCP `prepare_task` and `get_pack`.
- Add `--current-diff` to CLI `prepare-task` and `get-pack`.
- Keep the MCP tool surface unchanged.

## Non-goals

- Do not include patch hunks or source text.
- Do not run tests or shell validation from ctxpack.
- Do not make current diff automatic for every task; it remains explicit.

## Verification

- Unit test safe changed-path filtering and privacy status.
- MCP test that `prepare_task` can anchor changed files from the current diff.
- Full workspace tests, clippy, and CLI help smoke.
