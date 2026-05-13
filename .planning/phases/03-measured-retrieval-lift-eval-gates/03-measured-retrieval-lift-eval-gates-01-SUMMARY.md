---
phase: 03-measured-retrieval-lift-eval-gates
plan: 01
subsystem: retrieval-contracts
tags: [rust, serde-json, ctxpack-core, ctxpack-compiler, retrieval]

requires:
  - phase: 01-compatibility-guardrails-module-boundaries
    provides: Public JSON compatibility tests for additive serde contract changes
  - phase: 02-trust-layer-operational-diagnostics
    provides: Source-free diagnostics and compiler plan construction patterns
provides:
  - Additive typed retrieval candidate contracts on ContextPlan
  - Source-free attribution records on TargetFile and RelatedTest
  - Empty default wiring for current compiler plan construction
affects: [ctxpack-core, ctxpack-compiler, measured-retrieval-lift-eval-gates]

tech-stack:
  added: []
  patterns:
    - serde default vectors for additive backward-compatible fields
    - source-free evidence records using paths, roles, signal kinds, scores, edge labels, commit ids, counts, and reason codes

key-files:
  created:
    - .planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-01-SUMMARY.md
  modified:
    - crates/ctxpack-core/src/contracts.rs
    - crates/ctxpack-compiler/src/planning.rs
    - crates/ctxpack-compiler/src/lib.rs

key-decisions:
  - "Add retrievalCandidates and attribution as additive serde fields with default empty vectors to preserve old JSON compatibility."
  - "Expose typed candidate/evidence contracts now, but keep all ranking, pack, MCP, and eval behavior unchanged in Plan 01."
  - "Keep attribution source-free by omitting task text, source snippets, symbol signatures, and commit subject fields."

patterns-established:
  - "ContextPlan can carry typed retrievalCandidates while continuing to project targetFiles and relatedTests for existing clients."
  - "Constructors and compatibility fixtures initialize additive retrieval vectors explicitly with Vec::new()."

requirements-completed: [RETR-01, RETR-03, PARS-01]

duration: 10m05s
completed: 2026-05-13
---

# Phase 03 Plan 01: Measured Retrieval Lift Eval Gates Summary

**Typed, source-free retrieval candidate and attribution contracts added without changing current ranking behavior.**

## Performance

- **Duration:** 10m05s
- **Started:** 2026-05-13T14:48:43Z
- **Completed:** 2026-05-13T14:58:48Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added public serde contracts for `RetrievalCandidateKind`, `RetrievalSignalKind`, `RetrievalSignalScore`, `RetrievalEvidence`, and `RetrievalCandidate`.
- Added `targetFiles[].attribution`, `relatedTests[].attribution`, and `retrievalCandidates` as additive camelCase JSON fields with old-JSON defaults.
- Wired current compiler construction and test fixtures to emit empty vectors, preserving existing selection order and behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Add failing retrieval contract tests** - `801d690` (test)
2. **Task 1 GREEN: Add retrieval candidate contracts** - `baecdc5` (feat)
3. **Task 2: Wire additive retrieval defaults** - `d32f1ed` (fix)

## Files Created/Modified

- `crates/ctxpack-core/src/contracts.rs` - Adds typed retrieval candidates, signal scores, source-free evidence, additive attribution, and compatibility tests.
- `crates/ctxpack-compiler/src/planning.rs` - Initializes empty attribution on constructed targets/tests and empty `retrievalCandidates` on base plans.
- `crates/ctxpack-compiler/src/lib.rs` - Updates pack revalidation fixture target files with empty attribution.
- `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-01-SUMMARY.md` - Captures Plan 01 execution results.

## Decisions Made

- Used `#[serde(default)]` vectors for all additive fields so old plan JSON remains readable.
- Chose lowercase candidate-kind JSON values (`file`, `test`, `symbol`, `doc`, `commit`, `config`) and snake_case signal-kind JSON values to match existing enum conventions.
- Kept the new contracts declarative only; ranking and eval lift work remains for later Phase 3 plans.

## Deviations from Plan

None - plan executed exactly as scoped. No ranking behavior, pack snippets, MCP tools, eval behavior, dependencies, or parser/runtime changes were introduced.

## Issues Encountered

- Task 2 RED verification surfaced expected Rust compile errors for missing `attribution` and `retrieval_candidates` fields in compiler literals. The GREEN fix added explicit empty defaults at those construction points.

## Authentication Gates

None.

## Known Stubs

None. Stub scan found no `TODO`, `FIXME`, placeholder text, or hardcoded empty UI-facing data in created or modified files. The empty vectors are intentional additive contract defaults for current behavior.

## Verification

- `cargo test -p ctxpack-core retrieval -- --nocapture` passed.
- `cargo test -p ctxpack-core -p ctxpack-compiler -p ctxpack -- --nocapture` passed.
- `cargo fmt --all --check` passed.
- `cargo test --workspace` passed.
- `cargo run -p ctxpack -- --help` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 02 can build the internal ranking/projection layer on top of the typed contracts, attaching real source-free attribution while preserving the existing `ContextPlan` projection for CLI, MCP, packs, cards, and eval clients.

## Self-Check: PASSED

- Found files: `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-compiler/src/planning.rs`, `crates/ctxpack-compiler/src/lib.rs`, and this SUMMARY.
- Found commits via direct commit-object checks: `801d690`, `baecdc5`, `d32f1ed`.
- Verified no known stubs in created or modified files.

---
*Phase: 03-measured-retrieval-lift-eval-gates*
*Completed: 2026-05-13*
