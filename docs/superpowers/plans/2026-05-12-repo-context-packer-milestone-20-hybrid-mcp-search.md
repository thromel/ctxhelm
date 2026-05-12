# Milestone 20: Hybrid MCP Search

## Goal

Make the MCP `search` tool match the agent-native product contract by returning compact file and symbol matches from the local safe inventory.

## Scope

- Keep the tool local-only and read-only.
- Return source-free lexical file matches and symbol matches in one typed response.
- Add a `kinds` filter so agents can request only `file` or only `symbol` results.
- Preserve small bounded outputs through the existing `limit` guard.

## Non-goals

- No vector search, cloud reranking, hosted index, or source-code payloads.
- No expansion into test/history/dependency context; agents should use `related` for that.
- No new MCP tool names.

## Verification

- Focused MCP tests for default file+symbol search and `kinds: ["symbol"]`.
- Full workspace tests.
- Clippy with warnings denied.
- CLI help smoke after the MCP contract change.
