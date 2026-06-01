# ctxhelm Milestone 9: MCP Resources and Prompts

## Goal

Expose the remaining agent-native MCP surfaces beyond tools:

- resources for compact repo summaries, test maps, safe file slices, symbol search, and pack guidance
- prompts for bugfix, feature, refactor, review, test-writing, and explanation workflows

## Scope

- Add `resources/list` and `resources/read` handlers to `ctxhelm-mcp`.
- Add `prompts/list` and `prompts/get` handlers to `ctxhelm-mcp`.
- Keep outputs read-only and bounded.
- Prefer resource URIs and workflow instructions over large automatic context injection.

## Implemented Resources

- `ctxhelm://repo/summary`
- `ctxhelm://repo/test-map`
- `ctxhelm://pack/guide`
- `ctxhelm://file/<path>?lines=<start>-<end>`
- `ctxhelm://symbol/<query>`

## Implemented Prompts

- `bugfix`
- `feature`
- `refactor`
- `review_diff`
- `write_tests`
- `explain_area`

## Verification

- `cargo test -p ctxhelm-mcp --locked --offline`
- full workspace test, clippy, CLI help, and stdio MCP smoke are required before closing the milestone
