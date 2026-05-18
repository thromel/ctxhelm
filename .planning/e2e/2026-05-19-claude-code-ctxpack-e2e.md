# Claude Code ctxpack E2E Run

Date: 2026-05-19

## Scope

Verify that Claude Code can use ctxpack through the shipped MCP integration and
produce machine-checkable `prepare_task` and `get_pack` evidence with an
explicit `repo` argument.

## Environment

- ctxpack binary: `/tmp/ctxpack-claude-e2e/ctxpack-v1.1.0-aarch64-apple-darwin/ctxpack`
- ctxpack version: `ctxpack 1.1.0`
- Claude Code version: `2.1.143 (Claude Code)`
- repo: `/Users/romel/Documents/GitHub/Agent Memory`
- evidence: `/tmp/ctxpack-claude-e2e-evidence/claude-mcp-evidence.json`

## Command

```bash
CTXPACK_BIN=/tmp/ctxpack-claude-e2e/ctxpack-v1.1.0-aarch64-apple-darwin/ctxpack \
CTXPACK_REQUIRE_REAL_CLIENT=1 \
CTXPACK_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxpack-claude-e2e-evidence \
CTXPACK_ROOT=/Users/romel/Documents/GitHub/Agent\ Memory \
CTXPACK_SMOKE_REPO=/Users/romel/Documents/GitHub/Agent\ Memory \
CTXPACK_SMOKE_TASK='Use ctxpack to prepare a source-free context plan for improving release governance docs' \
CTXPACK_SMOKE_PATH='docs/release-governance.md' \
CTXPACK_SMOKE_QUERY='release governance' \
bash scripts/smoke-claude-mcp.sh
```

## Result

Passed.

Evidence payload:

```json
{
  "client": "claude",
  "clientVersion": "2.1.143 (Claude Code)",
  "ctxpackVersion": "ctxpack 1.1.0",
  "getPack": true,
  "prepareTask": true,
  "repo": "/Users/romel/Documents/GitHub/Agent Memory",
  "required": true
}
```

## Bugs And Findings

1. Documentation version drift: `README.md` and `docs/agent-setup.md` still
   described the local Claude Code evidence version as `2.1.140`. The actual
   e2e run used `2.1.143`. Fixed in this pass.
2. No ctxpack/Claude MCP runtime bug was observed. Claude Code started the
   strict MCP config and the server-side request evidence recorded both
   `prepare_task` and `get_pack` with the explicit repo path.
3. Diagnostic gap to consider later: `scripts/smoke-claude-mcp.sh` keeps only
   the compact evidence file on success. If we want richer post-run analysis,
   add an opt-in environment variable to preserve the Claude stream JSON and
   raw request log.

