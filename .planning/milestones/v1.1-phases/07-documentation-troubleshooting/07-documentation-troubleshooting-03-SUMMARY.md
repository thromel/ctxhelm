---
phase: 07-documentation-troubleshooting
plan: 03
subsystem: docs
tags: [troubleshooting, docs-check, release-gate, tdd]
requires:
  - phase: 07-documentation-troubleshooting
    provides: README quickstart and agent setup docs from Plans 01-02
provides:
  - Troubleshooting reference for install, PATH, MCP startup, CTXPACK_HOME, cleanup, wrong-cwd, setup-check, and pack resource issues
  - Expanded release docs checker covering README, release, quickstart, agent setup, and troubleshooting docs
  - Rust integration test contract for Phase 7 docs consistency checks
affects: [phase-08-release-gates, docs, packaging]
tech-stack:
  added: []
  patterns: [grep-based docs consistency gate, TDD docs checker contract]
key-files:
  created: [docs/troubleshooting.md]
  modified: [scripts/check-release-docs.sh, crates/ctxpack/tests/release_packaging.rs]
key-decisions:
  - "The docs gate remains a narrow grep-based shell check instead of a Markdown parser."
  - "Troubleshooting docs state setup-check validates repo-local artifacts only and does not run real agent clients."
patterns-established:
  - "Docs consistency checks reject unsupported Cursor/OpenCode real-client proof claims and source-checkout setup examples in normal-user docs."
requirements-completed: [DOCS-02, DOCS-04]
duration: 3min
completed: 2026-05-13
---

# Phase 07 Plan 03: Troubleshooting And Docs Gate Summary

**Troubleshooting reference plus automated docs consistency gate for Phase 8 release readiness**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-13T19:11:19Z
- **Completed:** 2026-05-13T19:14:10Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `docs/troubleshooting.md` covering PATH failures, absolute MCP binary paths, `CTXPACK_HOME`, uninstall/state cleanup, wrong cwd, MCP startup, stdout cleanliness, setup-check scope, and session-scoped pack resources.
- Added a failing Rust docs-gate contract first, then expanded `scripts/check-release-docs.sh` to satisfy it.
- Verified the checker, focused release docs test, CLI help, and full workspace tests.

## Task Commits

1. **Task 1: Add troubleshooting and state cleanup reference** - `7f99bd8` (docs)
2. **Task 2 RED: Expand docs consistency gate contract** - `5c08619` (test)
3. **Task 2 GREEN: Expand docs consistency checker** - `bd1a0e2` (docs)

## Files Created/Modified

- `docs/troubleshooting.md` - Operational troubleshooting and state cleanup reference.
- `scripts/check-release-docs.sh` - Docs consistency checks for Phase 7 docs and unsupported proof claims.
- `crates/ctxpack/tests/release_packaging.rs` - Integration-test contract for the docs checker.

## Decisions Made

- README link work required by this plan was already satisfied by Plan 01's `More Docs` section, so Plan 03 did not re-edit README.
- The shell checker remains intentionally grep-based and small; no Markdown parser or runtime behavior was added.

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None.

## Issues Encountered

- The troubleshooting verification expected lowercase `wrong cwd`; the heading was adjusted before the Task 1 commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 8 release gates can reuse `bash scripts/check-release-docs.sh`, `cargo test --workspace`, and `cargo run -p ctxpack -- --help` as documentation and release-readiness checks.

## Verification

- `python3 -c 'from pathlib import Path; d=Path("docs/troubleshooting.md").read_text(); required=["command not found","absolute","PATH","CTXPACK_HOME","~/.ctxpack","uninstall","state cleanup","wrong cwd","explicit `--repo`","MCP startup","stdout","setup-check","does not run real agent clients","session-scoped","get_pack"]; missing=[s for s in required if s not in d]; assert not missing, missing'`
- RED: `cargo test -p ctxpack --test release_packaging release_docs -- --nocapture` failed on missing `docs/quickstart.md` checker coverage before script changes.
- GREEN: `bash -n scripts/check-release-docs.sh && bash scripts/check-release-docs.sh && cargo test -p ctxpack --test release_packaging release_docs -- --nocapture`
- `bash scripts/check-release-docs.sh`
- `cargo test -p ctxpack --test release_packaging release_docs -- --nocapture`
- `cargo run -p ctxpack -- --help`
- `cargo test --workspace`

## Self-Check: PASSED

- Found created file: `docs/troubleshooting.md`
- Found modified file: `scripts/check-release-docs.sh`
- Found modified file: `crates/ctxpack/tests/release_packaging.rs`
- Found task commit: `7f99bd8`
- Found task commit: `5c08619`
- Found task commit: `bd1a0e2`

---
*Phase: 07-documentation-troubleshooting*
*Completed: 2026-05-13*
