---
phase: 03-measured-retrieval-lift-eval-gates
plan: 03
subsystem: retrieval-eval
tags: [rust, ctxhelm-index, ctxhelm-compiler, historical-eval, git]

requires:
  - phase: 03-measured-retrieval-lift-eval-gates
    provides: Fixed-budget candidate ranking and source-free attribution from Plans 01-02
  - phase: 02-trust-layer-operational-diagnostics
    provides: Safe inventory policy and source-free diagnostics
provides:
  - Status-aware historical commit samples from git name-status metadata
  - Source-free historical labels for safe, generated, sensitive, historical-only, rename, and delete paths
  - Frozen historical eval metadata with evalRangeId, budget, effective filters, refs, limit, mode, target agent, and repo id
affects: [ctxhelm-index, ctxhelm-compiler, ctxhelm-cli, retrieval-eval]

tech-stack:
  added: []
  patterns:
    - git diff-tree name-status -z -M parsing for historical labels
    - additive source-free JSON fields with safe compatibility projections
    - deterministic eval range ids derived from repo id, refs, budget, and filters

key-files:
  created:
    - .planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-03-SUMMARY.md
  modified:
    - crates/ctxhelm-index/src/git.rs
    - crates/ctxhelm-index/src/lib.rs
    - crates/ctxhelm-compiler/src/eval.rs
    - crates/ctxhelm-compiler/src/lib.rs
    - crates/ctxhelm/src/main.rs

key-decisions:
  - "Preserve safeChangedFiles as the compatibility projection while making changedPaths/changedPathLabels the rich source-free label surface."
  - "Keep HistoricalEvalOptions source-compatible in this plan and report the current fixed Standard budget as frozen eval metadata."
  - "Use parent-snapshot worktrees for replay and source-free path labels for truth, avoiding source snippets and commit subjects in serialized reports."

patterns-established:
  - "Historical commit sampling includes delete and excluded-path-only commits so label accounting does not silently drop negatives."
  - "Historical eval reports now carry explicit frozen metadata and per-commit role/status labels for downstream fixed-budget gates."

requirements-completed: [EVAL-01, EVAL-02, EVAL-05]

duration: 8m49s
completed: 2026-05-13
---

# Phase 03 Plan 03: Measured Retrieval Lift Eval Gates Summary

**Status-aware historical eval labels with frozen range metadata for reproducible fixed-budget retrieval metrics.**

## Performance

- **Duration:** 8m49s
- **Started:** 2026-05-13T15:17:02Z
- **Completed:** 2026-05-13T15:25:51Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Replaced path-only historical samples with `changedPaths` records from `git diff-tree --name-status -z -M`, including rename old paths and delete records.
- Preserved `safeChangedFiles` while labeling generated, sensitive, and historical-only paths without source text.
- Added `evalRangeId`, `budget`, `effectiveFilters`, `refs`, and per-commit `changedPathLabels` to historical eval reports.
- Updated CLI markdown rendering so human-readable historical reports show the frozen range, budget, and effective filters.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Add failing status-aware history tests** - `8d7536d` (test)
2. **Task 1 GREEN: Record status-aware historical labels** - `6a2936b` (feat)
3. **Task 2 RED: Add failing frozen eval tests** - `b4a00d1` (test)
4. **Task 2 GREEN: Freeze historical eval metadata** - `964493d` (feat)
5. **Rule 3 compile fix: Update CLI report fixture/rendering** - `0d70b3d` (fix)

## Files Created/Modified

- `crates/ctxhelm-index/src/git.rs` - Adds public rich historical label contracts, name-status parsing, rename/delete support, and bounded revision traversal.
- `crates/ctxhelm-index/src/lib.rs` - Re-exports index label contracts and adds history-sampling regressions.
- `crates/ctxhelm-compiler/src/eval.rs` - Adds frozen eval metadata and carries rich changed-path labels into commit eval records.
- `crates/ctxhelm-compiler/src/lib.rs` - Re-exports eval metadata types and adds historical eval report regressions.
- `crates/ctxhelm/src/main.rs` - Updates the CLI markdown report fixture/rendering for the new public eval fields.
- `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-03-SUMMARY.md` - Captures Plan 03 execution results.

## Decisions Made

- Kept `HistoricalEvalOptions` source-compatible because adding a required `budget` field would force broader CLI/API edits outside this plan. The report now freezes the current fixed `Standard` budget, and a later CLI-boundary plan can expose it as an option.
- Included commits with only deleted, generated, sensitive, or historical-only paths instead of filtering them out, so eval reports retain negative labels needed for gap analysis.
- Kept commit titles skipped from serialization; they remain available internally only to replay tasks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated CLI test fixture and markdown rendering for new public eval fields**
- **Found during:** Overall verification after Task 2
- **Issue:** `cargo test --workspace` failed because the CLI test fixture constructed `HistoricalEvalReport` and `HistoricalCommitEval` without the new public fields.
- **Fix:** Populated the new fixture fields and rendered frozen metadata in historical eval markdown output.
- **Files modified:** `crates/ctxhelm/src/main.rs`
- **Verification:** `cargo test --workspace` and `cargo run -p ctxhelm -- --help` passed.
- **Committed in:** `0d70b3d`

### Scope Adjustments

- `HistoricalEvalOptions` was not extended with a required budget field in this plan. This preserves source compatibility for existing Rust callers while still freezing the current `Standard` budget in report metadata.

---

**Total deviations:** 1 auto-fixed blocking issue, 1 compatibility-preserving scope adjustment.
**Impact on plan:** Historical eval reports are status-aware and reproducible; the remaining budget CLI configurability is deferred without blocking fixed-budget metrics.

## Issues Encountered

- Workspace validation caught downstream struct-literal fallout in the CLI crate after the compiler report gained public fields. The fix was narrow and did not change CLI arguments or command behavior.

## Authentication Gates

None.

## Known Stubs

None. Stub scan found no `TODO`, `FIXME`, placeholder text, source-less UI placeholders, or intentional empty mock data in created or modified files.

## Verification

- `cargo test -p ctxhelm-index historical_commit -- --nocapture` passed.
- `cargo test -p ctxhelm-compiler historical_eval -- --nocapture` passed.
- `cargo test -p ctxhelm-index -p ctxhelm-compiler` passed.
- `cargo test --workspace` passed.
- `cargo run -p ctxhelm -- --help` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 04 can consume status-aware labels and frozen range metadata for measured retrieval lift, gap reporting, and fixed-budget eval gates without relying on commit subjects or source snippets in reports.

## Self-Check: PASSED

- Found summary file at `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-03-SUMMARY.md`.
- Found task commits via direct commit-object checks: `8d7536d`, `6a2936b`, `b4a00d1`, `964493d`, `0d70b3d`.
- Verified no known stubs in created or modified files.
- Note: `git log --all` hit an existing malformed ref named `refs/heads/master 2`; direct commit-object checks and plain `git log -6` succeeded.

---
*Phase: 03-measured-retrieval-lift-eval-gates*
*Completed: 2026-05-13*
