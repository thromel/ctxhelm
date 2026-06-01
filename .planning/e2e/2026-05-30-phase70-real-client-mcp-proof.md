# Phase 70 E2E: Real-Client MCP Proof Refresh

Date: 2026-05-30

## Purpose

Refresh the optional real-client MCP proof after the Phase 69 channel-aware
product proof promoted default local retrieval. This proves that the promoted
agent-native path still works through actual Codex CLI and Claude Code clients,
not only through the deterministic JSON-RPC protocol smoke.

This is source-free client evidence. It proves tool-call wiring and explicit
repo handling; it does not claim Cursor or OpenCode real-client tool-call proof.

## Environment

- Repository: `/Users/romel/Documents/GitHub/Agent Memory`
- ctxhelm: `ctxhelm 1.1.0`
- Codex CLI: `codex-cli 0.130.0`
- Claude Code: `2.1.158 (Claude Code)`
- Evidence directory: `/tmp/ctxhelm-real-client-evidence`

## Commands

```bash
rm -rf /tmp/ctxhelm-real-client-evidence
mkdir -p /tmp/ctxhelm-real-client-evidence

CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxhelm-real-client-evidence \
CTXHELM_SMOKE_REPO="/Users/romel/Documents/GitHub/Agent Memory" \
bash scripts/smoke-codex-mcp.sh

CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxhelm-real-client-evidence \
CTXHELM_SMOKE_REPO="/Users/romel/Documents/GitHub/Agent Memory" \
bash scripts/smoke-claude-mcp.sh
```

## Results

Codex CLI:

```text
ctxhelm MCP protocol smoke ok: repo=/Users/romel/Documents/GitHub/Agent Memory anchor=crates/ctxhelm-mcp/src/lib.rs
ctxhelm Codex MCP smoke passed: server-side instrumentation recorded prepare_task and get_pack with repo=/Users/romel/Documents/GitHub/Agent Memory
```

Claude Code:

```text
ctxhelm MCP protocol smoke ok: repo=/Users/romel/Documents/GitHub/Agent Memory anchor=crates/ctxhelm-mcp/src/lib.rs
ctxhelm Claude MCP smoke passed: server-side instrumentation recorded prepare_task and get_pack with repo=/Users/romel/Documents/GitHub/Agent Memory
```

Machine-readable evidence:

```json
{"client": "codex", "clientVersion": "codex-cli 0.130.0", "ctxhelmVersion": "ctxhelm 1.1.0", "getPack": true, "prepareTask": true, "repo": "/Users/romel/Documents/GitHub/Agent Memory", "required": false}
{"client": "claude", "clientVersion": "2.1.158 (Claude Code)", "ctxhelmVersion": "ctxhelm 1.1.0", "getPack": true, "prepareTask": true, "repo": "/Users/romel/Documents/GitHub/Agent Memory", "required": false}
```

## Interpretation

- Codex CLI and Claude Code both exercised the real MCP client path.
- The server-side request logs observed both `prepare_task` and `get_pack`.
- Both calls carried the explicit repo path, so the smoke is not relying on the
  shell working directory.
- The hard deterministic protocol gate still runs before each real-client
  check.
- Cursor and OpenCode remain setup/protocol-proof integrations only in this
  release line; no machine-checkable real-client tool-call proof is claimed for
  them.

## Remaining Production-Readiness Work

This closes the immediate "real-client outcome proof" follow-up listed after
Phase 69. Remaining work should focus on protected evidence budget pressure,
parser/precision misses, docs/scripts storage gaps, and broader multi-repo
repeated-lift validation.
