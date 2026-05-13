---
phase: 08-release-gates-smoke-proof
plan: 03
subsystem: release
tags: [release-docs, docs-checker, release-gate, smoke-proof]
requires:
  - phase: 08-release-gates-smoke-proof
    provides: Plans 01 and 02 release gate and optional real-client wrappers
provides:
  - maintainer release gate documentation
  - docs consistency checks for release gate proof boundaries
  - final selected-binary release gate verification
affects: [release, documentation, smoke-proof]
tech-stack:
  added: []
  patterns: [grep-based docs gate, validation-only empty task commit]
key-files:
  created: []
  modified: [docs/release.md, scripts/check-release-docs.sh, crates/ctxpack/tests/release_packaging.rs]
key-decisions:
  - "Keep docs consistency as a narrow grep-based checker rather than adding a Markdown parser."
  - "Represent final release-gate validation with an empty task commit because Plan 01 already wired the optional hooks."
patterns-established:
  - "Release docs state required deterministic gates separately from optional real-client evidence."
  - "Docs checker rejects publish/tag/upload and unsupported Cursor/OpenCode real-client proof claims."
requirements-completed: [SMOKE-01, SMOKE-02, SMOKE-03, SMOKE-04]
duration: 3min
completed: 2026-05-13
---

# Phase 8 Plan 03: Release Gate Documentation Summary

**Maintainer docs and grep-based docs checks now describe and enforce the final local no-publish release gate**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-13T19:31:23Z
- **Completed:** 2026-05-13T19:34:06Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Documented `bash scripts/release-gate.sh` as the local pre-publication blocker.
- Documented selected-binary mode, extracted-binary fallback, required deterministic checks, optional Codex/Claude evidence, and no publish/tag/upload behavior.
- Extended `scripts/check-release-docs.sh` to require the final release gate strings and reject unsupported proof claims.
- Re-ran the final selected-binary release gate with `CTXPACK_SKIP_REAL_CLIENT=1`.

## Task Commits

1. **Task 1: Document release gate usage and proof boundaries** - `5bf2f47` (docs)
2. **Task 2: Expand docs checker for release gate consistency** - `2bad878` (test), `8adc312` (fix)
3. **Task 3: Wire optional real-client hooks into final gate and run full verification** - `6c8f9de` (test, empty validation commit)

## Files Created/Modified

- `docs/release.md` - Adds release gate instructions, required/optional checks, and no-publish boundaries.
- `scripts/check-release-docs.sh` - Requires release gate docs and rejects unsupported publication/client-proof claims.
- `crates/ctxpack/tests/release_packaging.rs` - Adds release docs checker contract expectations.

## Decisions Made

- Kept the docs checker grep-based, consistent with Phase 7.
- Did not add new release-gate code in Task 3 because Plan 01 already wired optional Codex/Claude hooks after deterministic proof.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Adjusted release docs wording to match exact gate contract**
- **Found during:** Task 1
- **Issue:** The verification expected the exact phrase `does not create tags`, while the first prose used `or create tags`.
- **Fix:** Reworded the release-gate boundary sentence to include the exact contract phrase.
- **Files modified:** `docs/release.md`
- **Verification:** Plan Task 1 Python docs assertion.
- **Committed in:** `5bf2f47`

**2. [Rule 3 - Blocking] Used an empty validation commit for a task with no remaining file changes**
- **Found during:** Task 3
- **Issue:** Plan 01 already wired optional real-client hooks into `scripts/release-gate.sh`, so Task 3 required verification but no new code.
- **Fix:** Ran the full selected-binary gate with `CTXPACK_SKIP_REAL_CLIENT=1` and recorded the validation task with an empty commit.
- **Files modified:** None
- **Verification:** `CTXPACK_BIN="$(pwd)/target/debug/ctxpack" CTXPACK_SKIP_REAL_CLIENT=1 bash scripts/release-gate.sh`
- **Committed in:** `6c8f9de`

**Total deviations:** 2 auto-fixed (Rule 1: 1, Rule 3: 1)
**Impact on plan:** No scope expansion; final gate behavior remained deterministic and no-publish.

## Issues Encountered

None blocking.

## User Setup Required

None - real Codex/Claude client proof remains opt-in through environment variables.

## Next Phase Readiness

Phase 8 is ready for final verification and milestone closeout.

## Self-Check: PASSED

- Found `docs/release.md`
- Found `scripts/check-release-docs.sh`
- Found `scripts/release-gate.sh`
- Found commits `5bf2f47`, `2bad878`, `8adc312`, and `6c8f9de`

---
*Phase: 08-release-gates-smoke-proof*
*Completed: 2026-05-13*
