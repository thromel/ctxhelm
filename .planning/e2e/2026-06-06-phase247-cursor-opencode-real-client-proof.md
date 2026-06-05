# Phase 247: Cursor/OpenCode Real-Client Proof Paths

**Date:** 2026-06-06
**Requirement:** AGENT-02
**Boundary:** source-free real-client request evidence only

## What Changed

- Added `scripts/smoke-cursor-real-client.sh`.
- Added `scripts/smoke-opencode-real-client.sh`.
- Wired both wrappers into `scripts/release-gate.sh`.
- Updated release docs, troubleshooting docs, governance docs, candidate status metadata, and release packaging contracts.
- Kept Cursor/OpenCode proof optional by default. Required mode is available through client-specific environment variables.
- Cursor real-client proof uses an isolated temporary Cursor workspace for `.cursor/mcp.json`; it does not write MCP config into the target repo or global config.

## Proof Contract

The deterministic MCP protocol gate still runs first. Real-client proof only passes when server-side instrumentation records both tool calls with the explicit target repo:

- `prepare_task`
- `get_pack`

Evidence files are source-free. They contain client/version metadata, request method counts, sanitized observed tool-call metadata, request-log hash/count, and a sanitized request-summary sidecar.

## Commands

```bash
bash -n scripts/smoke-cursor-real-client.sh scripts/smoke-opencode-real-client.sh
```

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
CTXHELM_REQUIRE_CURSOR_REAL_CLIENT=1 \
CTXHELM_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxhelm-agent02-cursor-required \
bash scripts/smoke-cursor-real-client.sh
```

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT=1 \
CTXHELM_REAL_CLIENT_TIMEOUT_SECONDS=20 \
CTXHELM_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxhelm-agent02-opencode-required \
bash scripts/smoke-opencode-real-client.sh
```

```bash
CTXHELM_ALLOW_DIRTY=1 \
CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT=1 \
CTXHELM_PROOF_DIR=/tmp/ctxhelm-phase247-proof \
bash scripts/release-gate.sh
```

## Results

- Syntax check passed.
- Cursor Agent CLI required proof failed honestly because `cursor agent status` reports not logged in.
- Cursor evidence path: `/tmp/ctxhelm-phase247-cursor-required/cursor-real-client-evidence.json`.
- Cursor evidence status: `skipped`.
- Cursor client version in required probe: `3.6.21`.
- Cursor explicit-repo tool-call count: `0`.
- OpenCode required proof passed.
- OpenCode evidence path: `/tmp/ctxhelm-phase247-opencode-required/opencode-real-client-evidence.json`.
- OpenCode client version: `1.14.25`.
- OpenCode evidence status: `passed`.
- OpenCode recorded MCP method counts: `initialize=1`, `notifications/initialized=1`, `tools/list=1`, `tools/call=2`.
- OpenCode explicit-repo tool-call count: `2`.
- OpenCode observed tool calls: `prepare_task` and `get_pack`, both with `repoMatched: true`.
- Release gate passed with `CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT=1`.
- Release proof summary: `/tmp/ctxhelm-phase247-proof-final/release-proof-summary.json`.
- Release proof optional statuses: `opencodeRealClientProof=passed`; `cursorRealClientProof=skipped:set CTXHELM_RUN_CURSOR_REAL_CLIENT=1 or CTXHELM_REQUIRE_CURSOR_REAL_CLIENT=1 after protocol gate passed`.

## Requirement Status

AGENT-02 is complete under its stated boundary: users can verify Cursor and OpenCode integration behavior where clients expose machine-checkable proof.

OpenCode currently provides a passing local source-free proof. Cursor has the same machine-checkable proof path, but this local machine cannot complete the required Cursor run until Cursor Agent CLI auth is available.
