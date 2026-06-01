---
phase: 03-measured-retrieval-lift-eval-gates
plan: 05
subsystem: testing
tags: [rust, cli, mcp, historical-eval, smoke-test, source-free]

# Dependency graph
requires:
  - phase: 03-measured-retrieval-lift-eval-gates
    provides: Phase 3 typed retrieval attribution, fixed-budget eval metrics, and source-free gap summaries from Plans 01-04
provides:
  - CLI compatibility coverage for retrievalCandidates, target/test attribution, and historical eval ranking metadata
  - MCP compatibility coverage proving prepare_task structuredContent exposes attributed recommendations without tool-surface changes
  - Bounded source-free historical eval smoke script with configurable repo, limit, and budget
affects: [cli-compatibility, mcp-compatibility, historical-eval, phase-3-validation]

# Tech tracking
tech-stack:
  added: [bash-smoke-script, python3-json-validation]
  patterns: [binary-cli-contract-tests, mcp-structuredContent-contract-tests, source-free-eval-smoke]

key-files:
  created:
    - scripts/smoke-historical-eval.sh
  modified:
    - crates/ctxhelm/tests/cli_compat.rs
    - crates/ctxhelm-mcp/src/lib.rs

key-decisions:
  - "Keep Plan 05 validation additive: tests assert new Phase 3 fields while preserving existing CLI/MCP keys and tool names."
  - "Use a source-free smoke script that validates fixed-budget report metadata instead of adding new runtime behavior."
  - "Represent the validation-only task with an empty task commit so the GSD per-task commit trail remains complete."

patterns-established:
  - "CLI compatibility tests now check additive retrieval attribution and eval ranking fields alongside legacy keys."
  - "Smoke validation reads JSON reports with python3 and rejects source/prompt text fields while allowing sourceTextLogged=false."

requirements-completed: [DIAG-03, RETR-05, EVAL-04, EVAL-05]

# Metrics
duration: 4m
completed: 2026-05-13
---

# Phase 03 Plan 05: Compatibility and Smoke Validation Summary

**CLI/MCP contract guards plus a bounded source-free historical eval smoke script for Phase 3 retrieval-lift validation**

## Performance

- **Duration:** 4m
- **Started:** 2026-05-13T15:42:52Z
- **Completed:** 2026-05-13T15:46:41Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added CLI compatibility assertions for `prepare-task` JSON retrieval candidates, target/test attribution, historical eval range metadata, fixed-budget ranking metrics, ablations, gap summaries, and `sourceTextLogged=false`.
- Added MCP compatibility assertions that `prepare_task` structuredContent exposes attributed target/test recommendations and retrieval candidates while the tool list remains unchanged.
- Added `scripts/smoke-historical-eval.sh`, a bounded source-free smoke path for the current repo and optional RefactoringMiner verification via `CTXHELM_REFACTORINGMINER_REPO`.
- Ran the full Phase 3 validation gate: workspace tests, CLI help, and bounded historical eval smoke.

## Task Commits

Each task was committed atomically:

1. **Task 1: Guard CLI and MCP additive compatibility** - `5f75eaf` (test)
2. **Task 2: Add bounded source-free historical eval smoke script** - `a08c092` (test)
3. **Task 3: Run final Phase 3 validation** - `4bcaec0` (test, empty validation commit)

## Files Created/Modified

- `crates/ctxhelm/tests/cli_compat.rs` - Extends binary CLI compatibility tests for additive retrieval/eval fields and source-free historical eval reports.
- `crates/ctxhelm-mcp/src/lib.rs` - Extends MCP prepare_task compatibility tests for attributed structuredContent without adding or changing tools.
- `scripts/smoke-historical-eval.sh` - Runs bounded `eval history` JSON smoke checks with configurable repo, limit, budget, and optional RefactoringMiner target.

## Decisions Made

- Kept compatibility checks additive and contract-focused; no public CLI keys, MCP tool names, resources, prompts, or runtime behavior were changed.
- Validated smoke reports by parsing JSON with `python3` so budget metadata and source-free guarantees fail fast in automation.
- Used an empty validation commit for Task 3 because the task intentionally had no file changes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired invalid local Git refs blocking summary self-check**
- **Found during:** Summary self-check
- **Issue:** `git log --all` failed on invalid local ref files named `.git/refs/heads/master 2` and `.git/refs/heads/master 3`.
- **Fix:** Moved the invalid ref files to `.git/invalid-refs-backup/` so Git could enumerate refs normally.
- **Files modified:** Local `.git` metadata only; no tracked project files.
- **Verification:** The required `git log --oneline --all | grep -q <hash>` checks found all task commits after the repair.
- **Committed in:** Not applicable; Git metadata repair is not tracked.

**Total deviations:** 1 auto-fixed (blocking local metadata repair)
**Impact on plan:** No code, CLI, MCP, script, or planning artifact behavior changed.

## Issues Encountered

- The first CLI compatibility assertion over-specified a private related-test attribution reason code. It was narrowed to the public contract shape: attribution evidence must expose typed `signal` and `reasonCode` fields.
- Summary self-check was initially blocked by invalid local Git ref files. The invalid refs were moved out of `.git/refs/heads`, then the required self-check passed.

## Auth Gates

None.

## Known Stubs

None.

## Verification

- `cargo test -p ctxhelm --test cli_compat -- --nocapture`
- `cargo test -p ctxhelm-mcp public_surface -- --nocapture`
- `cargo test -p ctxhelm-mcp prepare_task -- --nocapture`
- `CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_LIMIT=2 CTXHELM_SMOKE_BUDGET=10 bash scripts/smoke-historical-eval.sh`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
- `CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_LIMIT=2 CTXHELM_SMOKE_BUDGET=10 bash scripts/smoke-historical-eval.sh`

## Next Phase Readiness

Phase 3 has CLI, MCP, JSON, Markdown, and smoke coverage for the measured retrieval-lift surfaces. Phase 4 can build real client durability and restart/wrong-repo semantics on top of these stable contracts.

## Self-Check: PASSED

- `FOUND: scripts/smoke-historical-eval.sh`
- `FOUND: .planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-05-SUMMARY.md`
- `FOUND: 5f75eaf`
- `FOUND: a08c092`
- `FOUND: 4bcaec0`

---
*Phase: 03-measured-retrieval-lift-eval-gates*
*Completed: 2026-05-13*
