---
phase: 06-agent-setup-first-pack-adoption
plan: 02
subsystem: agent-adapters
tags: [agents, codex, claude, cursor, opencode, mcp]
requires:
  - phase: 06-agent-setup-first-pack-adoption
    provides: structured init reporting and first-run next-step output
provides:
  - Thin generated setup guidance for Codex, Claude Code, Cursor, and OpenCode
  - Regression coverage for explicit repo usage, native file reads, progressive get_pack, and MCP session scope
affects: [setup-check, first-pack-smoke, agent-native-adoption]
tech-stack:
  added: []
  patterns: [template contract tests, repo-local adapter snippets]
key-files:
  created: [.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-02-SUMMARY.md]
  modified: [crates/ctxhelm-core/src/init.rs]
key-decisions:
  - "Generated agent guidance must describe `prepare_task` resources as session-scoped and use `get_pack` as the durable reconnect path."
  - "Codex setup remains copy/paste-oriented and explicitly not applied by ctxhelm."
patterns-established:
  - "All adapter guidance artifacts share the same core contract: explicit repo, native reads, progressive pack materialization, session-scope caveat."
requirements-completed: [ADPT-02, ADPT-03]
duration: 2min
completed: 2026-05-13
---

# Phase 06 Plan 02: Agent Guidance Refresh Summary

**Thin generated adapter guidance for repo-explicit `prepare_task`, native reads, and durable `get_pack` fallback**

## Performance

- **Duration:** 2 min
- **Started:** 2026-05-13T18:45:00Z
- **Completed:** 2026-05-13T18:47:22Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Strengthened adapter guidance tests across AGENTS, Cursor, Claude command, OpenCode, and Codex setup text.
- Updated generated guidance to mention same-session/session-scoped MCP pack resources.
- Added durable reconnect guidance: call `get_pack` with the same task and `repo` after reconnect or restart.

## Task Commits

1. **Task 1: Tighten adapter guidance regression tests** - `eac06d2` (test)
2. **Task 2: Update generated Codex, Claude, Cursor, and OpenCode setup text** - `99679fa` (feat)

## Files Created/Modified

- `crates/ctxhelm-core/src/init.rs` - Adds adapter contract coverage and refreshes generated guidance constants.

## Decisions Made

- Generated text remains concise and dynamic; no repository maps, inventory dumps, source snippets, or static context are embedded.
- Absolute binary troubleshooting is documented as guidance only; ctxhelm does not resolve or write user-specific global paths.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p ctxhelm-core adapter -- --nocapture` - passed
- `cargo test -p ctxhelm-core init -- --nocapture` - passed
- `cargo test --workspace` - passed

## Self-Check: PASSED

- Summary file exists.
- Task commits verified in git history: `eac06d2`, `99679fa`.

---
*Phase: 06-agent-setup-first-pack-adoption*
*Completed: 2026-05-13*
