---
phase: 04-agent-native-client-durability
plan: 04
subsystem: testing
tags: [mcp, codex, claude, smoke-test, explicit-repo]

requires:
  - phase: 04-agent-native-client-durability
    provides: "Deterministic MCP protocol hard gate and session-scoped pack resource semantics"
provides:
  - "Optional Codex CLI real-client smoke wrapper layered on the protocol gate"
  - "Optional Claude Code real-client smoke wrapper layered on the protocol gate"
  - "Contract tests for wrapper syntax, required/skip controls, explicit repo prompts, and protocol-first ordering"
affects: [agent-native-client-durability, mcp, codex, claude, smoke-scripts]

tech-stack:
  added: []
  patterns:
    - "Real-client smokes first run the deterministic stdio MCP protocol gate"
    - "Server-side JSON-RPC instrumentation is accepted proof for client tool calls"
    - "Optional client mode skips unavailable/flaky clients while required mode fails without evidence"

key-files:
  created:
    - scripts/smoke-codex-mcp.sh
    - scripts/smoke-claude-mcp.sh
    - .planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-04-SUMMARY.md
  modified:
    - crates/ctxpack/tests/cli_compat.rs

key-decisions:
  - "Keep deterministic protocol smoke as the hard gate before any Codex or Claude attempt."
  - "Use isolated temp state for CTXPACK_HOME, Codex execution, Claude MCP config, and server-side request logs."
  - "Require machine-checkable prepare_task and get_pack calls with the explicit repo; final assistant prose is not proof."

patterns-established:
  - "Real-client smoke wrappers should report passed, skipped, or failed status explicitly."
  - "CTXPACK_REQUIRE_REAL_CLIENT=1 turns missing client evidence into a nonzero result."
  - "CTXPACK_SKIP_REAL_CLIENT=1 still runs the protocol hard gate before skipping client invocation."

requirements-completed: [AGNT-01]

duration: 6m11s
completed: 2026-05-13
---

# Phase 4 Plan 04: Real-Client MCP Smoke Wrappers Summary

**Codex and Claude MCP smoke wrappers with protocol-first gating and explicit-repo tool-call evidence checks**

## Performance

- **Duration:** 6m11s
- **Started:** 2026-05-13T16:36:21Z
- **Completed:** 2026-05-13T16:42:32Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `scripts/smoke-codex-mcp.sh`, which always runs `scripts/smoke-mcp-protocol.sh`, then attempts `codex exec` from an isolated temp config/state path unless `CTXPACK_SKIP_REAL_CLIENT=1`.
- Added `scripts/smoke-claude-mcp.sh`, which always runs the protocol gate, then attempts `claude -p` with a temp strict MCP config unless real-client mode is skipped.
- Added compatibility coverage proving both wrappers parse with `bash -n`, mention `prepare_task`, `get_pack`, and `repo`, support optional/required modes, and invoke the protocol gate before client execution.
- Implemented machine-checkable evidence validation by teeing client-to-server JSON-RPC requests and requiring both `prepare_task` and `get_pack` calls with the explicit repo.
- Closed the verification gap found after initial Phase 4 verification:
  - Codex `exec` now uses `--dangerously-bypass-approvals-and-sandbox` so non-interactive MCP calls are sent to the server instead of being auto-cancelled.
  - Claude smoke no longer uses `--bare`, so it exercises the normal authenticated Claude Code path instead of requiring `ANTHROPIC_API_KEY`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add script contract tests for real-client wrappers** - `2e64f0b` (test)
2. **Task 2: Implement Codex and Claude real-client smoke wrappers** - `abf1343` (feat)

_Note: Task 1 followed TDD RED first; Task 2 made the contract tests pass._

## Files Created/Modified

- `scripts/smoke-codex-mcp.sh` - Optional Codex CLI real-client wrapper with isolated `CODEX_HOME`, protocol-first execution, explicit repo prompt, and server-side evidence validation.
- `scripts/smoke-claude-mcp.sh` - Optional Claude Code real-client wrapper with temp MCP config, protocol-first execution, explicit repo prompt, and server-side evidence validation.
- `crates/ctxpack/tests/cli_compat.rs` - Adds wrapper syntax and contract guards that do not depend on client installation or auth.

## Decisions Made

- Use server-side request instrumentation as the durable proof mechanism because client JSON/event formats are less stable than the inbound JSON-RPC calls ctxpack receives.
- Keep real clients optional by default, but make `CTXPACK_REQUIRE_REAL_CLIENT=1` fail when installed clients cannot produce the required evidence.
- Keep all client configuration temporary and process-local; the wrappers do not mutate global Codex or Claude config.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Cargo-relative script path lookup in the contract test**
- **Found during:** Task 2 verification
- **Issue:** The new Rust test read `scripts/...` relative to the crate test process directory and failed even though the scripts existed at the workspace root.
- **Fix:** Resolve scripts from `CARGO_MANIFEST_DIR` up to the workspace root before reading and running `bash -n`.
- **Files modified:** `crates/ctxpack/tests/cli_compat.rs`
- **Verification:** `cargo test -p ctxpack --test cli_compat real_client_smoke_scripts -- --nocapture`
- **Committed in:** `abf1343`

**2. [Rule 1 - Bug] Added Claude stream-json verbosity flag**
- **Found during:** Default Claude real-client attempt
- **Issue:** The installed Claude CLI rejected `--output-format stream-json` without `--verbose`.
- **Fix:** Add `--verbose` to the Claude wrapper invocation.
- **Files modified:** `scripts/smoke-claude-mcp.sh`
- **Verification:** `CTXPACK_SMOKE_REPO="$PWD" bash scripts/smoke-claude-mcp.sh`
- **Committed in:** `abf1343`

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes were required for the planned wrapper and test contracts to run correctly. No scope expansion.

## Known Stubs

None. The `mcp_servers.ctxpack.args=[]` Codex config override is an intentional empty MCP server argument list, not placeholder data.

## Issues Encountered

- Initial verification found that Codex and Claude were installed, but optional default attempts did not produce machine-checkable `prepare_task` and `get_pack` evidence. Codex attempted the tool calls but cancelled them before server dispatch; Claude failed because `--bare` disables normal OAuth/keychain auth.
- The gap was fixed by enabling Codex non-interactive approval bypass for the isolated smoke and by removing Claude `--bare`. Required mode now passes for both clients with server-side JSON-RPC evidence.

## Verification

- `bash -n scripts/smoke-codex-mcp.sh`
- `bash -n scripts/smoke-claude-mcp.sh`
- `CTXPACK_SKIP_REAL_CLIENT=1 bash scripts/smoke-codex-mcp.sh`
- `CTXPACK_SKIP_REAL_CLIENT=1 bash scripts/smoke-claude-mcp.sh`
- `cargo test -p ctxpack --test cli_compat real_client_smoke_scripts -- --nocapture`
- `CTXPACK_SMOKE_REPO="$PWD" bash scripts/smoke-mcp-protocol.sh`
- `CTXPACK_SMOKE_REPO="$PWD" bash scripts/smoke-codex-mcp.sh`
- `CTXPACK_SMOKE_REPO="$PWD" bash scripts/smoke-claude-mcp.sh`
- `CTXPACK_REQUIRE_REAL_CLIENT=1 CTXPACK_SMOKE_REPO="$PWD" bash scripts/smoke-codex-mcp.sh` - passed with server-side `prepare_task` and `get_pack` evidence
- `CTXPACK_REQUIRE_REAL_CLIENT=1 CTXPACK_SMOKE_REPO="$PWD" bash scripts/smoke-claude-mcp.sh` - passed with server-side `prepare_task` and `get_pack` evidence
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo run -p ctxpack -- --help`

## User Setup Required

None for this machine; Codex CLI and Claude Code both produced real MCP tool-call evidence. Other machines still need authenticated client installs for required mode.

## Next Phase Readiness

Phase 4 now has deterministic MCP protocol coverage, bounded/session-scoped pack behavior, and real-client wrappers for Codex and Claude that pass required mode on this machine.

---
*Phase: 04-agent-native-client-durability*
*Completed: 2026-05-13*

## Self-Check: PASSED

- Found `scripts/smoke-codex-mcp.sh`
- Found `scripts/smoke-claude-mcp.sh`
- Found `crates/ctxpack/tests/cli_compat.rs`
- Found `.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-04-SUMMARY.md`
- Found commit `2e64f0b`
- Found commit `abf1343`
