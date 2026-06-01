---
phase: 03-measured-retrieval-lift-eval-gates
plan: 04
subsystem: eval
tags: [ctxhelm, historical-eval, ranking-metrics, ablations, retrieval-gaps]
requires:
  - phase: 03-measured-retrieval-lift-eval-gates
    provides: frozen historical eval ranges and role/status labels from Plan 03
provides:
  - fixed-budget combined-vs-lexical ranking metrics
  - signal ablation results over the same evalRangeId and commit count
  - source-free retrieval gap summaries grouped by role, signal gap, and path family
  - checklist rendering for grouped retrieval failures
affects: [phase-03, phase-04, eval-history, eval-checklist, retrieval-ranking]
tech-stack:
  added: []
  patterns:
    - source-free typed eval contracts with legacy compatibility projections
    - fixed-K metric computation over frozen commit samples
key-files:
  created:
    - .planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-04-SUMMARY.md
  modified:
    - crates/ctxhelm-compiler/src/eval.rs
    - crates/ctxhelm-compiler/src/lib.rs
    - crates/ctxhelm/src/main.rs
key-decisions:
  - "Keep legacy Recall@5/10 fields while adding decision-grade rankingComparison metrics."
  - "Expose signal ablations and grouped retrieval failures as source-free typed JSON fields."
  - "Wire checklist failure rendering through the same RetrievalGapSummary contract used by historical eval."
patterns-established:
  - "Historical eval reports compare combined and lexical rankings at the same configured K."
  - "Gap summaries use reason codes, roles, path families, counts, and example paths without prompt/source/commit-subject text."
requirements-completed: [DIAG-03, RETR-05, EVAL-03, EVAL-05, PARS-02, PARS-03]
duration: 9m56s
completed: 2026-05-13
---

# Phase 03 Plan 04: Ranking Metrics, Ablations, and Gap Reports Summary

**Historical retrieval eval now reports fixed-budget ranking quality, lexical lift, signal ablations, and source-free grouped retrieval failures.**

## Performance

- **Duration:** 9m56s
- **Started:** 2026-05-13T15:29:52Z
- **Completed:** 2026-05-13T15:39:48Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added `RankingMetrics`, `EvalComparison`, and role recall records so combined ctxhelm ranking and lexical-only ranking are compared at the same configured K.
- Added signal ablation results over the same `evalRangeId` and evaluated commit count, with parser/runtime dependency scope still gated by measured evidence.
- Added `RetrievalGapSummary` records and Markdown rendering for source-free grouped failures by path role, signal gap reason code, and repeated path family.
- Added `ctxhelm eval history --budget <K>` with a default of 10 and wired that fixed budget through JSON and Markdown outputs.
- Added checklist grouped retrieval failure rendering using the same typed `RetrievalGapSummary` contract as historical eval.

## Task Commits

1. **Task 1: Add fixed-budget ranking metrics and lexical comparison** - `1de0ae5` (feat)
2. **Task 2: Add signal ablations and grouped retrieval gap summaries** - `0ea3606` (feat)
3. **Task 3: Add checklist retrieval-failure summaries for DIAG-03** - `bdba79f` (feat)

## Files Created/Modified

- `crates/ctxhelm-compiler/src/eval.rs` - Adds ranking metrics, fixed-K comparison, signal ablations, and retrieval gap summaries.
- `crates/ctxhelm-compiler/src/lib.rs` - Re-exports the new eval contracts and adds focused TDD coverage.
- `crates/ctxhelm/src/main.rs` - Wires `eval history --budget`, renders ablations and grouped failures, and adds checklist gap rendering.
- `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-04-SUMMARY.md` - Execution summary and self-check record.

## Decisions Made

- Kept the existing legacy recall fields as compatibility projections while adding `rankingComparison` as the decision-grade surface.
- Used local typed structs and existing search/ranking data only; no Tantivy, rayon, SQLite/rusqlite, notify, tree-sitter, or MCP SDK dependency was added.
- Checklist output now uses the same typed retrieval gap records as historical eval instead of a separate string-only diagnostic format.

## Deviations from Plan

None - plan executed exactly as written.

## Auth Gates

None.

## Known Stubs

None.

## Verification

- `cargo test -p ctxhelm-compiler ranking_metrics -- --nocapture`
- `cargo test -p ctxhelm-compiler ablation -- --nocapture`
- `cargo test -p ctxhelm eval_checklist -- --nocapture`
- `cargo test -p ctxhelm historical_eval_report -- --nocapture`
- `cargo tree --workspace --depth 1 > /tmp/ctxhelm-phase3-cargo-tree.txt && ! rg "tantivy|rayon|rusqlite|notify|tree-sitter|mcp-sdk" /tmp/ctxhelm-phase3-cargo-tree.txt`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 05 can validate CLI/MCP compatibility and bounded historical-eval smokes against the new fixed-budget metrics, ablations, and grouped failure surfaces.

## Self-Check: PASSED

- Found modified implementation files and the Plan 04 summary file.
- Verified task commits `1de0ae5`, `0ea3606`, and `bdba79f` exist with `git cat-file -e`.

---
*Phase: 03-measured-retrieval-lift-eval-gates*
*Completed: 2026-05-13*
