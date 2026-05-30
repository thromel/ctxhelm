# Phase 102: Explicit-Repo MCP Resource Consumption

## Goal

Make source-free repo-scoped MCP resources consumable in the same launch shape
used by real coding agents: the MCP server may start outside the repository,
while the agent passes the active workspace through the `repo` tool argument.

## Problem

Phases 96-101 exposed context-area resources and made retrieval-gap summaries
point to them. Those resources still resolved their repository from the MCP
server cwd. In the release and real-client smokes, `prepare_task` and
`get_pack` pass an explicit repo from a non-repo server cwd. That meant an
agent could receive `ctxpack://repo/context-area/...` and fail to read it when
following the URI.

## Implementation

- Added an MCP session repo hint in `ctxpack-mcp`.
- `prepare_task`, `get_pack`, `search`, `related`, and `related_tests` remember
  the explicit repo they resolved.
- Repo-scoped resources first try the server cwd as before, then fall back to
  the last explicit repo when cwd discovery fails.
- `scripts/smoke-mcp-protocol.sh` now reads:
  - `ctxpack://repo/context-areas`
  - a dynamic `ctxpack://repo/context-area/{encoded-area}` resource for the
    anchor path
- The protocol smoke verifies `sourceTextLogged = false`, non-empty path counts,
  and non-empty `nextReadBatches`.
- Codex, Claude, Cursor, and OpenCode smoke evidence now records deterministic
  context-area resource-read coverage without claiming Cursor/OpenCode
  real-client tool-call transcripts.

## Validation

```bash
cargo test -p ctxpack-mcp repo_resources_use_last_explicit_tool_repo_when_server_cwd_is_not_repo -- --nocapture
cargo test -p ctxpack --test cli_compat -- --nocapture
CTXPACK_SMOKE_REPO="/Users/romel/Documents/GitHub/Agent Memory" \
  CTXPACK_SMOKE_TASK="verify release gate MCP protocol proof" \
  CTXPACK_SMOKE_PATH="crates/ctxpack-mcp/src/lib.rs" \
  CTXPACK_SMOKE_QUERY="prepare_task" \
  bash scripts/smoke-mcp-protocol.sh
CTXPACK_BIN="/Users/romel/Documents/GitHub/Agent Memory/target/debug/ctxpack" \
  bash scripts/smoke-cursor-mcp.sh
```

## Result

- Context-area MCP resources now work after explicit-repo tool calls from a
  non-repo server cwd.
- The deterministic protocol proof consumes the same resource class that
  resource-backed gap summaries return.
- Ranking, pack budgets, and source-free contracts are unchanged.
