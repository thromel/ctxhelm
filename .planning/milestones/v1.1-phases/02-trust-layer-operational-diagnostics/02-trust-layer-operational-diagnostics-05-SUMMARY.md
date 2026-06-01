---
phase: 02-trust-layer-operational-diagnostics
plan: 05
subsystem: trust-layer
tags: [rust, ctxhelm-cli, ctxhelm-mcp, diagnostics, trace-status, safe-resources]

requires:
  - phase: 02-trust-layer-operational-diagnostics
    provides: Plan 04 compiler diagnostics, pack revalidation, and card diagnostics
provides:
  - CLI prepare-task/get-pack diagnostics with non-fatal trace-write failures
  - Additive CLI --no-trace controls for read-oriented context commands
  - MCP tool diagnostics for plan, pack, search, related, related_tests, and current_diff read paths
  - MCP file resources revalidated through fresh safe inventory and read_safe_source
  - Full Phase 2 workspace validation
affects: [ctxhelm-cli, ctxhelm-mcp, ctxhelm-index, trust-layer-operational-diagnostics]

tech-stack:
  added: []
  patterns:
    - read-oriented trace writes use try_append_eval_trace and surface trace_write_failed diagnostics
    - CLI JSON object outputs expose additive diagnostics while array-shaped outputs keep compatibility
    - MCP related_tests keeps array-shaped structuredContent and exposes diagnostics as a result sibling for compatibility
    - MCP file resources use load_or_refresh_inventory plus read_safe_source immediately before returning text

key-files:
  created:
    - .planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-05-SUMMARY.md
  modified:
    - crates/ctxhelm-index/src/traces.rs
    - crates/ctxhelm-index/src/lib.rs
    - crates/ctxhelm/src/main.rs
    - crates/ctxhelm/tests/cli_compat.rs
    - crates/ctxhelm-mcp/src/tools.rs
    - crates/ctxhelm-mcp/src/resources.rs
    - crates/ctxhelm-mcp/src/schemas.rs
    - crates/ctxhelm-mcp/src/lib.rs

key-decisions:
  - "Trace recording remains on by default, with additive CLI --no-trace and MCP recordTrace controls for read-oriented commands."
  - "MCP related_tests preserves its existing array-shaped structuredContent and exposes diagnostics at the tool-result level to avoid a breaking shape change."
  - "MCP file resources now revalidate source paths against fresh safe inventory before reading source text."

patterns-established:
  - "Use try_append_eval_trace for prepare-task/get-pack style read paths and append returned diagnostics instead of failing the command."
  - "Use report variants from ctxhelm-index when MCP surfaces need diagnostics while retaining legacy result fields."
  - "Use read_safe_source for MCP file resources, not direct fs::read_to_string."

requirements-completed: [SAFE-01, SAFE-02, SAFE-04, SAFE-05, SAFE-06, DIAG-01, DIAG-02, DIAG-04]

duration: 9m57s
completed: 2026-05-13
---

# Phase 02 Plan 05: Trust Layer Operational Diagnostics Summary

**CLI and MCP read paths now surface structured diagnostics while trace writes and file-resource reads stay safe under constrained local state.**

## Performance

- **Duration:** 9m57s
- **Started:** 2026-05-13T14:03:48Z
- **Completed:** 2026-05-13T14:13:45Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Added `try_append_eval_trace` so `prepare-task`, `get-pack`, `prepare_task`, and `get_pack` continue returning context when trace storage cannot be written.
- Added `--no-trace` to CLI read commands and `recordTrace` to MCP read-tool schemas/arguments.
- Exposed diagnostics on CLI plan/pack JSON and MCP read-tool structured results while preserving existing command/tool names and compatibility shapes.
- Replaced MCP file-resource direct reads with fresh inventory revalidation plus `read_safe_source`.
- Completed the full Phase 2 verification sequence, including workspace tests and CLI help.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Add failing CLI diagnostics trace tests** - `d21d60d` (test)
2. **Task 1 GREEN: Make CLI trace writes non-fatal** - `bea78c2` (feat)
3. **Task 2 RED: Add failing MCP diagnostics tests** - `2cdbec7` (test)
4. **Task 2 GREEN: Expose MCP diagnostics safely** - `98a5d1d` (feat)
5. **Task 3 Validation: Validate Phase 2 diagnostics** - `e6fbdbc` (test)

## Files Created/Modified

- `crates/ctxhelm-index/src/traces.rs` - Added non-fatal trace append status and `trace_write_failed` diagnostics.
- `crates/ctxhelm-index/src/lib.rs` - Re-exported `try_append_eval_trace`.
- `crates/ctxhelm/src/main.rs` - Added CLI `--no-trace` controls and trace diagnostics projection for plan/pack JSON.
- `crates/ctxhelm/tests/cli_compat.rs` - Added binary tests for diagnostics fields, stale cache diagnostics, constrained `CTXHELM_HOME`, and no-trace help.
- `crates/ctxhelm-mcp/src/tools.rs` - Switched MCP read tools to diagnostics-aware report APIs and non-fatal trace writes.
- `crates/ctxhelm-mcp/src/resources.rs` - Revalidated file resources through fresh safe inventory and `read_safe_source`.
- `crates/ctxhelm-mcp/src/schemas.rs` - Added `recordTrace` schema controls for `prepare_task` and `get_pack`.
- `crates/ctxhelm-mcp/src/lib.rs` - Added MCP diagnostics, trace-write, and safe file-resource tests.

## Decisions Made

- Kept trace recording enabled by default to preserve existing behavior, with additive opt-out controls.
- Kept CLI array-shaped outputs unchanged; diagnostics are exposed on object-shaped plan/pack outputs.
- Kept MCP `related_tests` structured content array-shaped for compatibility and attached diagnostics as a result-level sibling.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Preserved MCP related warning text after diagnostics wiring**
- **Found during:** Task 2 full MCP verification
- **Issue:** Switching `related` to diagnostic report APIs changed the existing warning text for unavailable git co-change history.
- **Fix:** Restored the existing "co-change hints were unavailable" wording while retaining structured diagnostics.
- **Files modified:** `crates/ctxhelm-mcp/src/tools.rs`
- **Verification:** `cargo test -p ctxhelm-mcp -- --nocapture` passed.
- **Committed in:** `98a5d1d`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The fix preserved Phase 1 MCP compatibility while keeping the new diagnostics additive. No Phase 3 retrieval ranking, parser, or eval-gate scope was introduced.

## Issues Encountered

- The TDD RED tests intentionally failed for missing CLI/MCP diagnostics behavior before implementation.
- The MCP test environment lock now recovers from poisoned test locks so intentional RED failures do not cascade.

## Authentication Gates

None.

## Known Stubs

None. Stub scan found only the intentional MCP session-scoped pack-resource error text: "pack resource is not available in this MCP session; call prepare_task first".

## Verification

- `cargo test -p ctxhelm --test cli_compat -- --nocapture` passed.
- `cargo test -p ctxhelm-mcp diagnostics -- --nocapture` passed.
- `cargo test -p ctxhelm-mcp file_resource -- --nocapture` passed.
- `cargo test -p ctxhelm-mcp trace -- --nocapture` passed.
- `cargo test -p ctxhelm-mcp -- --nocapture` passed.
- `cargo test --workspace` passed.
- `cargo run -p ctxhelm -- --help` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 2 is complete. Phase 3 can start measured retrieval-lift work on top of fresh, safe, diagnostic-rich CLI and MCP boundaries.

## Self-Check: PASSED

- Found summary file and all key modified files.
- Found task commits via `git cat-file`: `d21d60d`, `bea78c2`, `2cdbec7`, `98a5d1d`, `e6fbdbc`.
- `git log --all` is blocked by a pre-existing malformed `refs/heads/master 2` ref, so commit existence was verified directly without modifying refs.
- Verified no known stubs in created or modified files.

---
*Phase: 02-trust-layer-operational-diagnostics*
*Completed: 2026-05-13*
