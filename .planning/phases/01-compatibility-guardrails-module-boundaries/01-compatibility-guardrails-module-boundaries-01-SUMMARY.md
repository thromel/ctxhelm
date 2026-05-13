---
phase: 01-compatibility-guardrails-module-boundaries
plan: 01
subsystem: testing
tags: [rust, cargo, cli, assert_cmd, compatibility]

requires: []
provides:
  - Binary-level CLI compatibility tests for ctxpack core commands
  - Real temp git repository and CTXPACK_HOME fixture helpers
  - Structured JSON shape assertions for CLI command outputs
affects: [phase-01, cli, compatibility, module-boundaries]

tech-stack:
  added: [assert_cmd, predicates]
  patterns:
    - Cargo integration tests under crates/ctxpack/tests
    - Command-local CTXPACK_HOME for binary test isolation
    - serde_json::Value shape assertions instead of full snapshots

key-files:
  created:
    - crates/ctxpack/tests/common/mod.rs
    - crates/ctxpack/tests/cli_compat.rs
  modified:
    - crates/ctxpack/Cargo.toml
    - Cargo.lock

key-decisions:
  - "Use assert_cmd::Command::cargo_bin for compiled ctxpack binary coverage."
  - "Assert stable JSON fields and selected values instead of snapshotting dynamic output."
  - "Use command-local CTXPACK_HOME and explicit --repo paths for isolation."

patterns-established:
  - "CLI integration tests create real committed repositories and avoid user-local ctxpack state."
  - "Compatibility tests parse JSON and check public camelCase keys without pinning UUIDs or hashes."

requirements-completed: [CONT-01]

duration: 6 min
completed: 2026-05-13
---

# Phase 01 Plan 01: CLI Compatibility Guardrails Summary

**Binary-level ctxpack CLI guardrails using real temp git repos, isolated CTXPACK_HOME, and structured JSON compatibility assertions**

## Performance

- **Duration:** 6 min
- **Started:** 2026-05-13T12:00:14Z
- **Completed:** 2026-05-13T12:06:35Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `assert_cmd` and `predicates` test support for compiled `ctxpack` binary tests.
- Added shared CLI integration-test fixtures that create real git repos, fixture source/test/config files, generated and sensitive exclusions, and isolated `CTXPACK_HOME`.
- Added `cli_compat` coverage for `--help`, `index`, `prepare-task`, `get-pack`, `search`, `related-tests`, `dependencies`, `eval history`, and `serve-mcp`.
- Verified command outputs through structured JSON shape checks, local write side effects, explicit `--repo` paths, and newline-delimited JSON-RPC responses.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add CLI integration-test dependencies and fixture helpers** - `2704f0d` (test)
2. **Task 2: Cover core CLI commands through the compiled binary** - `cf7be80` (test)

**Plan metadata:** pending final docs commit

## Files Created/Modified

- `crates/ctxpack/Cargo.toml` - Added CLI integration-test dev dependencies.
- `Cargo.lock` - Locked test-only dependencies required by `assert_cmd` and `predicates`.
- `crates/ctxpack/tests/common/mod.rs` - Added temp git repo, isolated home, git runner, and JSON stdout helpers.
- `crates/ctxpack/tests/cli_compat.rs` - Added binary-level compatibility tests for the core CLI command surface.

## Decisions Made

- Used `assert_cmd::Command::cargo_bin("ctxpack")` so tests exercise the compiled binary and Clap wiring.
- Used structured `serde_json::Value` assertions for JSON output shape and selected stable values.
- Kept dynamic values such as repo IDs, task hashes, UUIDs, and ordering loosely asserted to avoid brittle snapshots.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added tempfile as a ctxpack dev dependency**
- **Found during:** Task 1 (Add CLI integration-test dependencies and fixture helpers)
- **Issue:** The planned helper exposes `tempfile::TempDir`, but `tempfile` was only a workspace dependency and not available to the `ctxpack` integration-test crate.
- **Fix:** Added `tempfile.workspace = true` under `crates/ctxpack` dev-dependencies.
- **Files modified:** `crates/ctxpack/Cargo.toml`, `Cargo.lock`
- **Verification:** `cargo test -p ctxpack --no-run`
- **Committed in:** `2704f0d`

**2. [Rule 3 - Blocking] Added a fixture dependency edge**
- **Found during:** Task 2 (Cover core CLI commands through the compiled binary)
- **Issue:** The base fixture needed a real local import for the `dependencies` command to assert `sourcePath`, `targetPath`, and `kind` on a non-empty result.
- **Fix:** Added `src/auth/token.ts` and an import from `src/auth/session.ts` in the shared fixture.
- **Files modified:** `crates/ctxpack/tests/common/mod.rs`
- **Verification:** `cargo test -p ctxpack --test cli_compat`
- **Committed in:** `cf7be80`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were test-infrastructure requirements needed to satisfy the planned binary guardrails. No runtime behavior changed.

## Issues Encountered

- Initial TDD run of `cli_compat` exposed two over-specific assertions: repo IDs are current hash-like values rather than `repo-*`, and lexical search may rank the related test before the source file. Assertions were corrected to preserve the current public contract without pinning unstable details.

## Known Stubs

None.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p ctxpack --no-run` passed.
- `cargo test -p ctxpack --test cli_compat` passed.
- `cargo run -p ctxpack -- --help` passed.
- `cargo test --workspace` passed.

## Next Phase Readiness

Plan 01 now satisfies `CONT-01`. Maintainers can run the binary CLI compatibility target before public contract or module-boundary work in later Phase 01 plans.

## Self-Check: PASSED

- Created/modified Plan 01 files exist on disk.
- Task commits `2704f0d` and `cf7be80` exist and resolve as Git commit objects.
- Verification commands listed above completed successfully.

---
*Phase: 01-compatibility-guardrails-module-boundaries*
*Completed: 2026-05-13*
