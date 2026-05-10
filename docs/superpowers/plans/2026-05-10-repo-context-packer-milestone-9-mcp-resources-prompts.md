# Repo Context Packer Milestone 9: MCP Resources and Prompts

## Goal

Expose the remaining agent-native MCP surfaces beyond tools:

- resources for compact repo summaries, test maps, safe file slices, symbol search, and pack guidance
- prompts for bugfix, feature, refactor, review, test-writing, and explanation workflows

## Scope

- Add `resources/list` and `resources/read` handlers to `ctxpack-mcp`.
- Add `prompts/list` and `prompts/get` handlers to `ctxpack-mcp`.
- Keep outputs read-only and bounded.
- Prefer resource URIs and workflow instructions over large automatic context injection.

## Implemented Resources

- `ctxpack://repo/summary`
- `ctxpack://repo/test-map`
- `ctxpack://pack/guide`
- `ctxpack://file/<path>?lines=<start>-<end>`
- `ctxpack://symbol/<query>`

## Implemented Prompts

- `bugfix`
- `feature`
- `refactor`
- `review_diff`
- `write_tests`
- `explain_area`

## Verification

- `cargo test -p ctxpack-mcp --locked --offline`
- full workspace test, clippy, CLI help, and stdio MCP smoke are required before closing the milestone
