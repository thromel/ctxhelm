---
phase: 06-agent-setup-first-pack-adoption
plan: 03
subsystem: cli
tags: [setup-check, validation, init, agents]
requires:
  - phase: 06-agent-setup-first-pack-adoption
    provides: structured init report and refreshed adapter guidance
provides:
  - Read-only `ctxpack setup-check` command
  - Typed setup validation report for generated files, JSON snippets, and thin guidance
  - CLI status rendering with pass, warn, and fail outcomes
affects: [first-pack-smoke, release-gates, troubleshooting-docs]
tech-stack:
  added: [serde_json as normal ctxpack-core dependency]
  patterns: [typed validation reports, non-mutating CLI checks]
key-files:
  created: [.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-03-SUMMARY.md]
  modified: [crates/ctxpack-core/Cargo.toml, crates/ctxpack-core/src/init.rs, crates/ctxpack-core/src/lib.rs, crates/ctxpack/src/main.rs, crates/ctxpack/tests/cli_compat.rs]
key-decisions:
  - "`setup-check` validates repo-local generated artifacts and never runs or mutates real agent clients."
  - "Warnings do not fail setup-check; missing or malformed expected artifacts do."
patterns-established:
  - "Core validation returns structured pass/warn/fail items and the CLI decides process exit."
  - "JSON adapter snippets are syntax-checked without requiring them to carry full workflow prose."
requirements-completed: [ADPT-04]
duration: 5min
completed: 2026-05-13
---

# Phase 06 Plan 03: Setup Check Summary

**Read-only `ctxpack setup-check` validation for generated setup artifacts and adapter snippets**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-13T18:47:22Z
- **Completed:** 2026-05-13T18:52:34Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `SetupCheckReport`, `SetupCheckItem`, and `SetupCheckStatus` contracts.
- Implemented read-only validation for AGENTS, ctxpack config, Cursor, Claude, and OpenCode generated artifacts.
- Exposed `ctxpack setup-check` with adapter flags, help text, pass/warn/fail output, and non-zero failure exit.

## Task Commits

1. **Task 1: Add core setup validation report** - `bd0bdda` (test), `8c1089b` (feat), `241c43c` (fix)
2. **Task 2: Expose `ctxpack setup-check` through the CLI** - `6881dbe` (test), `d6479a3` (feat)

## Files Created/Modified

- `crates/ctxpack-core/Cargo.toml` - Promotes existing workspace `serde_json` dependency for production JSON validation.
- `crates/ctxpack-core/src/init.rs` - Adds setup-check report types, validator, and tests.
- `crates/ctxpack-core/src/lib.rs` - Re-exports setup-check public contracts.
- `crates/ctxpack/src/main.rs` - Adds `setup-check` subcommand and renderer.
- `crates/ctxpack/tests/cli_compat.rs` - Adds setup-check CLI compatibility tests.

## Decisions Made

- `setup-check` reports absolute-path/PATH guidance as a warning item so users see it without failing otherwise valid setup.
- JSON snippets are checked for size, forbidden static-context phrases, and JSON syntax; full prose contract checks stay on the generated guidance artifacts.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Declared production JSON parsing dependency**
- **Found during:** Task 2 CLI test build
- **Issue:** `ctxpack-core` used `serde_json` in `run_setup_check`, but `serde_json` was only a dev-dependency.
- **Fix:** Moved the existing workspace `serde_json` dependency into normal `ctxpack-core` dependencies.
- **Files modified:** `crates/ctxpack-core/Cargo.toml`
- **Verification:** `cargo test -p ctxpack --test cli_compat setup_check -- --nocapture`, `cargo test --workspace`
- **Committed in:** `241c43c`

---

**Total deviations:** 1 auto-fixed (Rule 3)
**Impact on plan:** Required to compile the new read-only validation path; no new external dependency was introduced beyond the existing workspace dependency.

## Issues Encountered

None beyond the dependency classification fix above.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p ctxpack-core setup_check -- --nocapture` - passed
- `cargo test -p ctxpack --test cli_compat setup_check -- --nocapture` - passed
- `cargo run -p ctxpack -- init --repo "$repo" --cursor --claude --opencode` - passed
- `cargo run -p ctxpack -- setup-check --repo "$repo" --cursor --claude --opencode` - passed
- `cargo run -p ctxpack -- --help` - passed
- `cargo test --workspace` - passed

## Self-Check: PASSED

- Summary file exists.
- Task commits verified in git history: `bd0bdda`, `8c1089b`, `241c43c`, `6881dbe`, `d6479a3`.

---
*Phase: 06-agent-setup-first-pack-adoption*
*Completed: 2026-05-13*
