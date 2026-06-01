# Milestone 8: Complete MCP Tool Surface

## Goal

Expose the full small MCP tool surface promised by the MVP roadmap while keeping ctxhelm local-first and read-only.

## Scope

1. Keep `prepare_task` and `get_pack` repo-aware.
2. Add MCP `search` backed by safe lexical repository search.
3. Add MCP `related` backed by related-test detection and git co-change hints.
4. Add MCP `related_tests` backed by targeted test mapping.
5. Add MCP `current_diff` backed by git path lists, without returning source content.
6. Add fixture-backed tests and an end-to-end stdio smoke test.

## Out of Scope

- MCP resources.
- MCP prompts.
- Remote MCP.
- Write/edit/run-command tools.

## Verification

- `cargo fmt --all --check`
- `cargo test --workspace --locked --offline`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo run -p ctxhelm -- --help`
- Stdio MCP smoke test for `tools/list`, `search`, `related`, `related_tests`, and `current_diff`.
