---
phase: 08-release-gates-smoke-proof
plan: 01
subsystem: release
tags: [release-gate, smoke-test, packaging, mcp, binary]
requires:
  - phase: 05-release-identity-binary-packaging
    provides: release package and artifact audit scripts
  - phase: 06-agent-setup-first-pack-adoption
    provides: first-pack smoke script
  - phase: 07-documentation-troubleshooting
    provides: release docs checker
provides:
  - local release gate script for deterministic pre-publication proof
  - release gate contract tests covering no-publish boundaries
affects: [release, smoke-proof, docs]
tech-stack:
  added: []
  patterns: [bash orchestrator over existing release smoke scripts, selected-binary proof via CTXHELM_BIN]
key-files:
  created: [scripts/release-gate.sh]
  modified: [crates/ctxhelm/tests/release_packaging.rs]
key-decisions:
  - "Run optional real-client wrappers from the release gate, but skip real-client auth by default unless CTXHELM_REQUIRE_REAL_CLIENT=1."
  - "Use scripts/release-package.sh as the single source for package/audit behavior, then prove either the selected CTXHELM_BIN or the extracted archive binary."
patterns-established:
  - "Release gate steps are named and fail fast."
  - "Binary-facing smoke scripts receive the same canonical CTXHELM_BIN."
requirements-completed: [SMOKE-01, SMOKE-02, SMOKE-04]
duration: 4min
completed: 2026-05-13
---

# Phase 8 Plan 01: Core Release Gate Summary

**Local release gate that proves package/audit output, selected or extracted binary identity, first-pack behavior, and wrong-cwd MCP protocol behavior without publishing**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-13T19:24:42Z
- **Completed:** 2026-05-13T19:28:20Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `scripts/release-gate.sh` as the maintainer-facing local release gate.
- Added contract coverage proving required component scripts, selected-binary propagation, extracted-binary fallback, and no publish/tag/upload behavior.
- Verified the gate with `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm"`; real-client wrappers were skipped by default after deterministic protocol proof.

## Task Commits

1. **Task 1: Add release gate script contract tests** - `f8e114e` (test)
2. **Task 2: Implement installed-binary release gate orchestration** - `dd29dda` (feat)

## Files Created/Modified

- `scripts/release-gate.sh` - Orchestrates workspace tests, docs consistency, release packaging/audit, binary identity checks, first-pack smoke, MCP protocol smoke, and optional real-client wrappers.
- `crates/ctxhelm/tests/release_packaging.rs` - Adds deterministic release-gate script contract coverage.

## Decisions Made

- Optional real-client wrappers are part of the final gate path, but default to skipped unless explicitly required. This keeps local release readiness deterministic on unprovisioned machines.
- The gate runs `scripts/release-package.sh` even when `CTXHELM_BIN` is supplied so artifact audit stays in the release path; selected binary proof still uses the provided executable for runtime smoke checks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Defaulted real-client gate hooks to skipped unless required**
- **Found during:** Task 2
- **Issue:** Invoking optional Codex/Claude wrappers by default could require local auth/client state, contrary to the user constraint that real-client checks remain opt-in.
- **Fix:** `scripts/release-gate.sh` now passes `CTXHELM_SKIP_REAL_CLIENT=1` by default unless `CTXHELM_REQUIRE_REAL_CLIENT=1`.
- **Files modified:** `scripts/release-gate.sh`
- **Verification:** `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" bash scripts/release-gate.sh`
- **Committed in:** `dd29dda`

**2. [Rule 1 - Bug] Removed premature Plan 03 docs-checker assertions from Plan 01**
- **Found during:** Task 2 full gate verification
- **Issue:** Workspace tests failed because Plan 01 added release-doc checker expectations before Plan 03 updates the checker and docs.
- **Fix:** Kept Plan 01 tests scoped to release-gate behavior; moved docs-checker expansion to Plan 03.
- **Files modified:** `crates/ctxhelm/tests/release_packaging.rs`
- **Verification:** `cargo test -p ctxhelm --test release_packaging release_gate -- --nocapture`
- **Committed in:** `dd29dda`

**Total deviations:** 2 auto-fixed (Rule 1: 1, Rule 2: 1)
**Impact on plan:** Kept the release gate aligned with the no-auth-by-default requirement and preserved ordered phase execution.

## Issues Encountered

The first full release-gate run failed on a premature docs-checker contract assertion. The assertion was deferred to Plan 03 where the docs checker is actually updated.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 02 can harden the optional Codex/Claude wrappers that the release gate now invokes.

## Self-Check: PASSED

- Found `scripts/release-gate.sh`
- Found `crates/ctxhelm/tests/release_packaging.rs`
- Found commits `f8e114e` and `dd29dda`

---
*Phase: 08-release-gates-smoke-proof*
*Completed: 2026-05-13*
