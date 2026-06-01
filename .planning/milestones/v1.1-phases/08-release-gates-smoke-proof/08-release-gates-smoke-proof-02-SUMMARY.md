---
phase: 08-release-gates-smoke-proof
plan: 02
subsystem: release
tags: [codex, claude, mcp, smoke-test, evidence]
requires:
  - phase: 04-agent-native-client-durability
    provides: deterministic MCP protocol proof and real-client smoke patterns
  - phase: 06-agent-setup-first-pack-adoption
    provides: optional real-client smoke wrappers
provides:
  - versioned source-free Codex MCP smoke evidence contract
  - versioned source-free Claude MCP smoke evidence contract
affects: [release-gate, client-smoke, docs]
tech-stack:
  added: []
  patterns: [selected-binary MCP server wrappers, JSON evidence files gated by env]
key-files:
  created: []
  modified: [scripts/smoke-codex-mcp.sh, scripts/smoke-claude-mcp.sh, crates/ctxhelm/tests/cli_compat.rs]
key-decisions:
  - "Real-client execution is env-gated by CTXHELM_RUN_REAL_CLIENT=1 or CTXHELM_REQUIRE_REAL_CLIENT=1; CTXHELM_SKIP_REAL_CLIENT=1 still forces a deterministic-only pass."
  - "Evidence JSON includes client, clientVersion, ctxhelmVersion, repo, prepareTask, getPack, and required without source snippets or prompt text."
patterns-established:
  - "Optional client wrappers always run deterministic protocol proof before considering real clients."
  - "Selected CTXHELM_BIN is used for protocol smoke, version capture, and stdio serve-mcp wrappers."
requirements-completed: [SMOKE-03]
duration: 3min
completed: 2026-05-13
---

# Phase 8 Plan 02: Real-Client Evidence Summary

**Optional Codex and Claude smoke wrappers now produce versioned machine-checkable MCP evidence when explicitly run, while remaining deterministic-only by default**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-13T19:28:20Z
- **Completed:** 2026-05-13T19:31:23Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Hardened Codex and Claude smoke wrappers to resolve and validate selected `CTXHELM_BIN`.
- Captured exact client version and selected ctxhelm version before real-client execution.
- Added stable source-free evidence output through `CTXHELM_REAL_CLIENT_EVIDENCE_DIR`.
- Verified deterministic skip mode for both wrappers without requiring auth.

## Task Commits

1. **Task 1: Add real-client evidence contract tests** - `ba98357` (test)
2. **Task 2: Record versioned real-client smoke evidence** - `73bfba8` (feat)

## Files Created/Modified

- `scripts/smoke-codex-mcp.sh` - Uses selected binary for protocol and MCP server execution, records Codex evidence when opted in.
- `scripts/smoke-claude-mcp.sh` - Uses selected binary for protocol and MCP server execution, records Claude evidence when opted in.
- `crates/ctxhelm/tests/cli_compat.rs` - Adds real-client smoke wrapper contract checks.

## Decisions Made

- Added `CTXHELM_RUN_REAL_CLIENT=1` as a non-required opt-in path, while preserving `CTXHELM_REQUIRE_REAL_CLIENT=1` as the required evidence gate.
- Real-client wrappers keep temporary logs private unless `CTXHELM_REAL_CLIENT_EVIDENCE_DIR` is set for stable evidence output.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Made real-client execution opt-in instead of opportunistic**
- **Found during:** Task 2
- **Issue:** The prior wrappers attempted real clients whenever `codex` or `claude` existed, which could force auth/model behavior on default release checks.
- **Fix:** Real-client execution now requires `CTXHELM_RUN_REAL_CLIENT=1` or `CTXHELM_REQUIRE_REAL_CLIENT=1`; deterministic protocol proof still runs first.
- **Files modified:** `scripts/smoke-codex-mcp.sh`, `scripts/smoke-claude-mcp.sh`
- **Verification:** `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-codex-mcp.sh` and Claude equivalent.
- **Committed in:** `73bfba8`

**Total deviations:** 1 auto-fixed (Rule 2: 1)
**Impact on plan:** Aligns implementation with the user constraint that real-client checks stay opt-in by default.

## Issues Encountered

None beyond the opt-in real-client boundary adjustment.

## User Setup Required

None - Codex/Claude auth is not required by default. Maintainers can opt in with `CTXHELM_RUN_REAL_CLIENT=1` or require proof with `CTXHELM_REQUIRE_REAL_CLIENT=1`.

## Next Phase Readiness

Plan 03 can document the release gate and extend docs checks to cover the final proof boundaries.

## Self-Check: PASSED

- Found `scripts/smoke-codex-mcp.sh`
- Found `scripts/smoke-claude-mcp.sh`
- Found `crates/ctxhelm/tests/cli_compat.rs`
- Found commits `ba98357` and `73bfba8`

---
*Phase: 08-release-gates-smoke-proof*
*Completed: 2026-05-13*
