---
phase: 06-agent-setup-first-pack-adoption
plan: 04
subsystem: scripts
tags: [first-pack, smoke, mcp, adoption, binary]
requires:
  - phase: 06-agent-setup-first-pack-adoption
    provides: setup-check command and refreshed adapter guidance
provides:
  - `CTXHELM_BIN` support for deterministic MCP protocol smoke
  - `scripts/smoke-first-pack.sh` install-to-first-pack adoption smoke
  - CI/local guard tests for MCP protocol and first-pack script contracts
affects: [release-gates, troubleshooting-docs, installed-binary-validation]
tech-stack:
  added: []
  patterns: [selected-binary smoke scripts, temp repo adoption fixture, machine-checkable JSON validation]
key-files:
  created: [.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-04-SUMMARY.md, scripts/smoke-first-pack.sh]
  modified: [scripts/smoke-mcp-protocol.sh, crates/ctxhelm/tests/cli_compat.rs]
key-decisions:
  - "First-pack smoke uses `CTXHELM_BIN` or `command -v ctxhelm`, not cargo, for the user-facing path."
  - "Deterministic MCP protocol proof remains the hard gate; real Codex/Claude clients stay optional."
patterns-established:
  - "Adoption smokes should use explicit `repo` arguments and validate structured JSON rather than assistant prose."
requirements-completed: [ADPT-05]
duration: 4min
completed: 2026-05-13
---

# Phase 06 Plan 04: First-Pack Smoke Summary

**Installed-binary first-pack smoke from init to setup-check to MCP protocol to plan and pack JSON**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-13T18:52:34Z
- **Completed:** 2026-05-13T18:56:29Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Updated `scripts/smoke-mcp-protocol.sh` to honor `CTXHELM_BIN` while preserving cargo fallback.
- Added `scripts/smoke-first-pack.sh`, which creates a temp git repo, runs init/setup-check, proves MCP, and validates prepare-task/get-pack JSON.
- Added script-contract and execution tests for the MCP and first-pack smoke paths.

## Task Commits

1. **Task 1: Let deterministic MCP smoke use a selected binary** - `54c3e2e` (test), `88e29aa` (feat)
2. **Task 2: Add first-pack quickstart smoke script** - `6324d62` (test), `068bfa4` (feat)

## Files Created/Modified

- `scripts/smoke-mcp-protocol.sh` - Supports selected installed/debug binary via `CTXHELM_BIN`.
- `scripts/smoke-first-pack.sh` - Runs the end-to-end first-pack adoption smoke.
- `crates/ctxhelm/tests/cli_compat.rs` - Guards script contracts and executes first-pack smoke with the cargo-built binary.

## Decisions Made

- The first-pack script validates machine-readable JSON fields (`targetFiles`, `packOptions`, `repoId`, `sections`) and avoids printing source snippets.
- Real Codex/Claude client smokes remain outside this deterministic adoption script because they depend on auth/client state.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p ctxhelm --test cli_compat mcp_protocol -- --nocapture` - passed
- `cargo test -p ctxhelm --test cli_compat first_pack -- --nocapture` - passed
- `cargo build -p ctxhelm && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" bash scripts/smoke-first-pack.sh` - passed
- `cargo test --workspace` - passed
- `cargo run -p ctxhelm -- --help` - passed

## Self-Check: PASSED

- Summary file exists.
- Task commits verified in git history: `54c3e2e`, `88e29aa`, `6324d62`, `068bfa4`.

---
*Phase: 06-agent-setup-first-pack-adoption*
*Completed: 2026-05-13*
