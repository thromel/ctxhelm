# Claude Code ctxhelm E2E Run

Date: 2026-05-19

## Scope

Verify that Claude Code can use ctxhelm through the shipped MCP integration and
produce machine-checkable `prepare_task` and `get_pack` evidence with an
explicit `repo` argument.

## Environment

- ctxhelm binary: `/tmp/ctxhelm-claude-e2e/ctxhelm-v1.1.0-aarch64-apple-darwin/ctxhelm`
- ctxhelm version: `ctxhelm 1.1.0`
- Claude Code version: `2.1.143 (Claude Code)`
- repo: `/Users/romel/Documents/GitHub/Agent Memory`
- evidence: `/tmp/ctxhelm-claude-e2e-evidence/claude-mcp-evidence.json`

## Command

```bash
CTXHELM_BIN=/tmp/ctxhelm-claude-e2e/ctxhelm-v1.1.0-aarch64-apple-darwin/ctxhelm \
CTXHELM_REQUIRE_REAL_CLIENT=1 \
CTXHELM_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxhelm-claude-e2e-evidence \
CTXHELM_ROOT=/Users/romel/Documents/GitHub/Agent\ Memory \
CTXHELM_SMOKE_REPO=/Users/romel/Documents/GitHub/Agent\ Memory \
CTXHELM_SMOKE_TASK='Use ctxhelm to prepare a source-free context plan for improving release governance docs' \
CTXHELM_SMOKE_PATH='docs/release-governance.md' \
CTXHELM_SMOKE_QUERY='release governance' \
bash scripts/smoke-claude-mcp.sh
```

## Result

Passed.

Evidence payload:

```json
{
  "client": "claude",
  "clientVersion": "2.1.143 (Claude Code)",
  "ctxhelmVersion": "ctxhelm 1.1.0",
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
2. No ctxhelm/Claude MCP runtime bug was observed. Claude Code started the
   strict MCP config and the server-side request evidence recorded both
   `prepare_task` and `get_pack` with the explicit repo path.
3. Diagnostic gap to consider later: `scripts/smoke-claude-mcp.sh` keeps only
   the compact evidence file on success. If we want richer post-run analysis,
   add an opt-in environment variable to preserve the Claude stream JSON and
   raw request log.

