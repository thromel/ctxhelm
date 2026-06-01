---
phase: 02-trust-layer-operational-diagnostics
plan: 03
subsystem: trust-layer
tags: [rust, ctxhelm-index, freshness, source-read-policy, diagnostics]

requires:
  - phase: 02-trust-layer-operational-diagnostics
    provides: Plan 02 trusted inventory freshness and Plan 01 source-read policy contracts
provides:
  - Freshness-aware index read paths for search, symbols, related tests, dependency graph, current diff, co-change, and history helpers
  - Diagnostic-bearing report variants for downstream compiler/MCP integration
  - Source-read diagnostics for non-UTF-8, oversized, deleted-after-inventory, and policy-skipped index inputs
  - Partial coverage diagnostics for graph, test-map, and git-history signals
affects: [ctxhelm-index, ctxhelm-compiler, ctxhelm-mcp, trust-layer-operational-diagnostics]

tech-stack:
  added: []
  patterns:
    - legacy vector-returning APIs delegate to diagnostic report variants where compatibility permits
    - read_safe_source gates content reads before scoring/parsing
    - report APIs degrade git failures into diagnostics while legacy co_change_hints preserves error compatibility

key-files:
  created:
    - .planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-03-SUMMARY.md
  modified:
    - crates/ctxhelm-index/src/lib.rs
    - crates/ctxhelm-index/src/search.rs
    - crates/ctxhelm-index/src/symbols.rs
    - crates/ctxhelm-index/src/related_tests.rs
    - crates/ctxhelm-index/src/dependencies.rs
    - crates/ctxhelm-index/src/git.rs
    - crates/ctxhelm-index/src/policy.rs

key-decisions:
  - "Index read APIs keep existing result shapes while exposing diagnostic report variants for downstream compiler and MCP wiring."
  - "Search, symbols, related tests, and dependency graph parse only content returned by read_safe_source."
  - "New git report APIs convert missing or timed-out git into diagnostics, while legacy co_change_hints still errors for existing compiler risk-flag compatibility."

patterns-established:
  - "Use load_or_refresh_inventory before inventory-derived user-facing index results."
  - "Aggregate source-read diagnostics into report-level diagnostics without embedding source text."
  - "Emit graph_partial, test_map_partial, and history_partial when lower-level signals degrade."

requirements-completed: [SAFE-01, SAFE-02, SAFE-05, DIAG-01, DIAG-04]

duration: 13m34s
completed: 2026-05-13
---

# Phase 02 Plan 03: Trust Layer Operational Diagnostics Summary

**Fresh index reads with source-safe diagnostics for stale inventory, read failures, and partial external signals.**

## Performance

- **Duration:** 13m34s
- **Started:** 2026-05-13T13:32:55Z
- **Completed:** 2026-05-13T13:46:29Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Routed search, symbol extraction/search, related tests, test map, dependency graph, current diff, co-change, and history helpers through `load_or_refresh_inventory`.
- Added diagnostic-bearing report APIs such as `lexical_search_report`, `extract_symbols_report`, `related_tests_report`, `dependency_edges_report`, `current_diff_summary_report`, and `historical_commit_samples_report`.
- Replaced content-scoring/parsing reads in search, symbols, tests, and dependency graph with `read_safe_source`.
- Added deterministic fixtures for stale-created files, non-UTF-8 files, oversized source-like files, deleted-after-inventory files, graph/test partial coverage, and missing git history.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Route index retrieval through trusted inventory** - `89c2b62` (test)
2. **Task 1 GREEN: Route index reads through fresh inventory** - `054f0b4` (feat)
3. **Task 2 RED: Replace silent source read failures with diagnostics** - `846e655` (test)
4. **Task 2 GREEN: Surface source read diagnostics** - `2cb92c9` (feat)
5. **Task 2 compatibility auto-fix** - `b6de6ec` (fix)

## Files Created/Modified

- `crates/ctxhelm-index/src/lib.rs` - Added freshness/source-read/partial-signal tests and re-exported report APIs.
- `crates/ctxhelm-index/src/search.rs` - Added `SearchReport`, fresh inventory loading, and safe source reads for lexical scoring.
- `crates/ctxhelm-index/src/symbols.rs` - Added symbol report APIs and parse-gap diagnostics for skipped source reads.
- `crates/ctxhelm-index/src/related_tests.rs` - Added related-test report APIs and test-map partial diagnostics.
- `crates/ctxhelm-index/src/dependencies.rs` - Added dependency report APIs and graph partial diagnostics.
- `crates/ctxhelm-index/src/git.rs` - Added report APIs for co-change, current diff, and history diagnostics.
- `crates/ctxhelm-index/src/policy.rs` - Added shared `SOURCE_READ_MAX_BYTES`.
- `.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-03-SUMMARY.md` - This execution summary.

## Decisions Made

- Kept legacy result-vector APIs intact and added report variants rather than changing existing return types.
- Preserved retrieval scoring and ordering logic; freshness and diagnostics are the only intentional behavior changes.
- Preserved legacy `co_change_hints` error behavior because compiler tests project that into `co_change_unavailable` risk flags.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Preserved legacy co-change error compatibility**
- **Found during:** Task 2 workspace verification
- **Issue:** The first diagnostic implementation made `co_change_hints` degrade missing git to an empty result, which broke existing compiler risk-flag behavior.
- **Fix:** Kept diagnostic degradation on `co_change_hints_report` while restoring strict error behavior for legacy `co_change_hints`.
- **Files modified:** `crates/ctxhelm-index/src/git.rs`
- **Verification:** `cargo test -p ctxhelm-compiler prepare_context_plan -- --nocapture` and `cargo test --workspace` passed.
- **Committed in:** `b6de6ec`

**2. [Rule 3 - Blocking] Cleared stale Cargo incremental artifact**
- **Found during:** Overall workspace verification
- **Issue:** `cargo test --workspace` failed creating an incremental dependency graph file under `target/debug/incremental` after parallel focused cargo runs.
- **Fix:** Ran `cargo clean -p ctxhelm-mcp` to remove generated build artifacts, then reran workspace tests.
- **Files modified:** None, generated build output only.
- **Verification:** `cargo test --workspace` passed.
- **Committed in:** Not applicable; generated artifact cleanup only.

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes preserved compatibility and verification reliability. No retrieval ranking, eval metric, cloud, or autonomous-editing scope was added.

## Issues Encountered

Initial workspace validation exposed the compatibility regression above and a generated Cargo incremental artifact error. Both were resolved before state updates.

## Authentication Gates

None.

## Known Stubs

None. Stub scan found no task-blocking stubs; matches for `placeholder` were existing test-script helper names, not placeholder functionality.

## Verification

- `cargo test -p ctxhelm-index lexical_search -- --nocapture` passed.
- `cargo test -p ctxhelm-index symbol -- --nocapture` passed.
- `cargo test -p ctxhelm-index related_tests -- --nocapture` passed.
- `cargo test -p ctxhelm-index dependency -- --nocapture` passed.
- `cargo test -p ctxhelm-index current_diff -- --nocapture` passed.
- `cargo test -p ctxhelm-index source_read -- --nocapture` passed.
- `cargo test -p ctxhelm-index diagnostics -- --nocapture` passed.
- `cargo test -p ctxhelm-index partial -- --nocapture` passed.
- `cargo test -p ctxhelm-index` passed: 44 tests plus doctests.
- `cargo test --workspace` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 04 can consume report diagnostics in compiler plan construction and pack/card source revalidation without changing index ranking behavior.

## Self-Check: PASSED

- Found summary file and key modified code files.
- Found task commits via direct commit-object checks: `89c2b62`, `054f0b4`, `846e655`, `2cb92c9`, `b6de6ec`.
- Verified no known stubs block the plan goal.

---
*Phase: 02-trust-layer-operational-diagnostics*
*Completed: 2026-05-13*
