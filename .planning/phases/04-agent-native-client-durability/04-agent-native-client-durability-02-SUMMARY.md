---
phase: 04-agent-native-client-durability
plan: 02
subsystem: agent-adapters
tags: [ctxpack, agents-md, mcp, adapters, init]
requires:
  - phase: 03-measured-retrieval-lift-eval-gates
    provides: measured attributed prepare_task and get_pack context surfaces
provides:
  - thin generated adapter guidance for AGENTS, Cursor, Claude, OpenCode, and Codex
  - regression tests preventing static dump wording and oversized generated guidance
  - progressive prepare_task then get_pack instructions with explicit repo guidance
affects: [phase-04-client-durability, adapter-generation, init-guidance]
tech-stack:
  added: []
  patterns: [small generated guidance fixtures, adapter phrase guards, byte-size guards]
key-files:
  created:
    - .planning/phases/04-agent-native-client-durability/deferred-items.md
  modified:
    - crates/ctxpack-core/src/init.rs
key-decisions:
  - "Keep generated adapter text as concise runtime guidance rather than static repository context."
  - "Tell agents to call prepare_task first with explicit repo, then request get_pack progressively only when direct file reads or brief context are insufficient."
patterns-established:
  - "Generated guidance tests cover forbidden static-dump phrases, byte-size limits, explicit repo usage, and progressive pack loading."
requirements-completed: [AGNT-04]
duration: 3m
completed: 2026-05-13
---

# Phase 04 Plan 02: Thin Dynamic Adapter Guidance Summary

**Generated AGENTS and native adapter guidance now stays small, repo-explicit, and points agents from dynamic `prepare_task` calls to progressive `get_pack` use.**

## Performance

- **Duration:** 3m
- **Started:** 2026-05-13T16:19:57Z
- **Completed:** 2026-05-13T16:22:51Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added adapter regression guards for static-dump phrases, size caps, dynamic `prepare_task` guidance, and explicit `repo` usage.
- Refreshed AGENTS, Cursor, Claude, OpenCode, and Codex wording to tell agents to use `get_pack` progressively only when direct file reads or brief context are insufficient.
- Removed outdated prepare_task-only wording from OpenCode/Codex guidance without changing runtime MCP behavior or adding dependencies.

## Task Commits

1. **Task 1 RED: Add adapter guidance guards** - `8cf7edd` (test)
2. **Task 1 GREEN: Require repo guidance in adapters** - `4628c82` (fix)
3. **Task 2 RED: Add progressive pack guidance guards** - `2860e44` (test)
4. **Task 2 GREEN: Refresh adapter pack guidance** - `0ef62cf` (fix)

_TDD tasks intentionally produced test then fix commits._

## Files Created/Modified

- `crates/ctxpack-core/src/init.rs` - Added adapter guidance tests and refreshed generated AGENTS/Cursor/Claude/OpenCode/Codex wording.
- `.planning/phases/04-agent-native-client-durability/deferred-items.md` - Recorded out-of-scope failures found during broad verification.

## Decisions Made

- Kept adapter guidance thin and dynamic instead of embedding static maps, inventories, snippets, or generated context.
- Kept `prepare_task` as the first dynamic call and positioned `get_pack` as progressive fallback context after native file reads.
- Preserved Claude MCP JSON as a small local stdio snippet rather than mutating global client configuration.

## Deviations from Plan

None - plan implementation stayed within `crates/ctxpack-core/src/init.rs` and planning docs. No runtime MCP behavior or dependency changes were made.

## Issues Encountered

- `cargo test --workspace` failed in `crates/ctxpack/tests/cli_compat.rs` on `mcp_protocol_uses_explicit_repo_from_wrong_cwd`. This file is outside Plan 02 ownership and was already modified by parallel Plan 01 work, so it was documented in `deferred-items.md` instead of fixed here.
- `cargo fmt --all --check` reported formatting diffs in out-of-scope files. These were also documented in `deferred-items.md` and left untouched.

## Verification

- `cargo test -p ctxpack-core adapter -- --nocapture` - passed
- `cargo test -p ctxpack-core init -- --nocapture` - passed
- `cargo test --workspace` - failed out of scope in `crates/ctxpack/tests/cli_compat.rs`
- `cargo fmt --all --check` - failed out of scope in `crates/ctxpack/tests/cli_compat.rs` and `crates/ctxpack-mcp/src/lib.rs`

## Known Stubs

None. Stub-pattern scan found only quoted diagnostic text in `deferred-items.md`, not implementation stubs or placeholder adapter content.

## Self-Check: PASSED

- Created/modified files exist: `crates/ctxpack-core/src/init.rs`, `deferred-items.md`, and this summary.
- Task commits exist: `8cf7edd`, `4628c82`, `2860e44`, `0ef62cf`.
- Forbidden static-dump and prepare_task-only phrases appear only inside regression-test assertions, not generated adapter content.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03 can rely on generated adapter guidance being small and dynamic while it hardens MCP pack resource and cache behavior. Plan 01 or the MCP durability follow-up should resolve the recorded `cli_compat.rs` workspace failure.

---
*Phase: 04-agent-native-client-durability*
*Completed: 2026-05-13*
