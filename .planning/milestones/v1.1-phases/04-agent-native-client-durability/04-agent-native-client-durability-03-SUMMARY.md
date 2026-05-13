---
phase: 04-agent-native-client-durability
plan: 03
subsystem: mcp
tags: [mcp, resources, session-scope, cache, diagnostics]

requires:
  - phase: 04-agent-native-client-durability
    provides: "Deterministic MCP protocol smoke with explicit repo arguments and same-session pack resource validation"
provides:
  - "Explicit MCP-session scoped pack resource guide and missing-resource diagnostics"
  - "Subprocess restart regression proving pack URIs do not survive a new serve-mcp process"
  - "Deterministically bounded in-memory pack resource cache with newest URI readability"
affects: [agent-native-client-durability, mcp, pack-resources, cache-growth]

tech-stack:
  added: []
  patterns:
    - "Session-scoped MCP pack resources stay process-local but return reconnect-safe diagnostics"
    - "Private insertion-order tracking bounds cache growth without adding an MCP tool"

key-files:
  created:
    - .planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-03-SUMMARY.md
  modified:
    - crates/ctxpack-mcp/src/resources.rs
    - crates/ctxpack-mcp/src/lib.rs
    - crates/ctxpack/tests/cli_compat.rs

key-decisions:
  - "Preserve process-local MCP pack resources and make the session boundary explicit instead of adding persistence or reconstruction."
  - "Use get_pack as the durable reconnect-safe materialization path after MCP client reconnects or server restarts."
  - "Bound pack resource cache growth privately with deterministic oldest-key eviction and test-only inspection helpers."

patterns-established:
  - "Restart behavior for pack resources should be tested through two separate ctxpack serve-mcp subprocess invocations."
  - "Long-running MCP cache behavior should be covered by in-process tests without widening the public MCP tool surface."

requirements-completed: [AGNT-02, AGNT-03]

duration: 4m20s
completed: 2026-05-13
---

# Phase 4 Plan 03: Pack Resource Session Scope and Cache Bound Summary

**Session-scoped MCP pack resources with reconnect diagnostics and deterministic bounded cache eviction**

## Performance

- **Duration:** 4m20s
- **Started:** 2026-05-13T16:28:45Z
- **Completed:** 2026-05-13T16:33:05Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Updated the pack guide and missing-pack error path so users can see that `ctxpack://pack/...` URIs are MCP-session scoped and only valid in the same server process.
- Added a binary subprocess restart regression that creates a pack URI in one `ctxpack serve-mcp` process, then proves a second process returns the clear session-scoped diagnostic.
- Bounded the in-memory pack resource cache with deterministic oldest-key eviction while keeping the newest generated URI readable.

## Task Commits

1. **Task 1 RED: Characterize restart and session-scoped diagnostics** - `d1c7408` (test)
2. **Task 1 GREEN: Clarify pack resource session scope** - `c10dca6` (fix)
3. **Task 2 RED: Add failing pack cache bound test** - `a5b30eb` (test)
4. **Task 2 GREEN: Bound MCP pack resource cache** - `afb5286` (fix)

## Files Created/Modified

- `crates/ctxpack-mcp/src/resources.rs` - Adds explicit session diagnostics, a richer pack guide, private cache limit, insertion-order tracking, and deterministic eviction.
- `crates/ctxpack-mcp/src/lib.rs` - Adds in-process tests for guide wording, missing-resource diagnostics, cache bounds, eviction, and newest URI readability.
- `crates/ctxpack/tests/cli_compat.rs` - Adds subprocess restart coverage for session-scoped pack URI behavior.
- `.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-03-SUMMARY.md` - Records execution results.

## Decisions Made

- Kept pack resources process-local and session-scoped to preserve the existing MCP behavior without introducing persistence, reconstruction metadata, or new dependencies.
- Directed reconnect/restart users to `get_pack` for durable pack materialization, or to `prepare_task` for fresh session resource URIs.
- Chose a private fixed cache size with oldest-key eviction instead of a cache-inspection MCP tool, preserving the public tool/resource surface.

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None.

## Issues Encountered

- The initial RED runs failed on the intended missing guide and diagnostic wording.
- A formatting check found one rustfmt wrap in `resources.rs`; the formatting change was amended into the Task 2 implementation commit and final verification was rerun afterward.

## Verification

- `cargo fmt --all --check`
- `cargo test -p ctxpack-mcp pack_resource -- --nocapture`
- `cargo test -p ctxpack-mcp pack_resource_cache -- --nocapture`
- `cargo test -p ctxpack --test cli_compat pack_resource -- --nocapture`
- `cargo test --workspace`
- `cargo run -p ctxpack -- --help`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03 closes the process-local pack-resource durability concern with explicit diagnostics and cache bounds. Plan 04 can focus on optional real Codex CLI and Claude Code smoke wrappers without changing MCP pack resource semantics.

## Self-Check: PASSED

- Found `crates/ctxpack-mcp/src/resources.rs`
- Found `crates/ctxpack-mcp/src/lib.rs`
- Found `crates/ctxpack/tests/cli_compat.rs`
- Found `.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-03-SUMMARY.md`
- Found commit `d1c7408`
- Found commit `c10dca6`
- Found commit `a5b30eb`
- Found commit `afb5286`

---
*Phase: 04-agent-native-client-durability*
*Completed: 2026-05-13*
