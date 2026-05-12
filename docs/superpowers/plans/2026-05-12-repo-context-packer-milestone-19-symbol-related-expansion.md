# Milestone 19: Symbol-Based Related Expansion

## Goal

Let agents use MCP `related` around a symbol as well as a file path. This closes the gap between symbol search resources and graph/test/history expansion.

## Scope

- Extend the existing `related` MCP tool input with optional `symbol`.
- Resolve symbol queries through the local safe symbol index.
- Expand related tests, dependency edges, and co-change hints around resolved symbol paths.
- Return `resolvedPaths` and `symbolMatches` for traceable evidence.
- Keep co-change failures non-fatal so symbol/path expansion still returns tests and dependency edges when git history is unavailable.
- Keep the MCP tool list unchanged.

## Non-Goals

- No new CLI command.
- No cloud symbol search.
- No LSP/SCIP precision references yet.
- No source content in the `related` response.

## Verification

- MCP unit test for symbol anchor expansion.
- MCP unit test for missing path/symbol validation.
- MCP unit test for graceful degradation when git history is unavailable.
- Full workspace tests, clippy, CLI help smoke, and live MCP smoke.
