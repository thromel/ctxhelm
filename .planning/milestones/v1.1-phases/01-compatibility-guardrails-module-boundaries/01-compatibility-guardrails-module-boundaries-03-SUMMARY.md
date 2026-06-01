---
phase: 01-compatibility-guardrails-module-boundaries
plan: 03
subsystem: indexing
tags: [rust, ctxhelm-index, module-boundaries, compatibility]

requires:
  - phase: 01-compatibility-guardrails-module-boundaries
    provides: CLI, JSON, and MCP compatibility guardrails from Plans 01 and 02
provides:
  - Focused ctxhelm-index modules behind stable crate-root exports
  - Inventory, search, symbols, related-tests, dependencies, git/history/current-diff, and trace boundaries
  - Verified downstream compiler, MCP, and CLI compatibility after the split
affects: [ctxhelm-index, ctxhelm-compiler, ctxhelm-mcp, ctxhelm-cli]

tech-stack:
  added: []
  patterns: [crate-root facade, private concern modules, public re-exports]

key-files:
  created:
    - crates/ctxhelm-index/src/inventory.rs
    - crates/ctxhelm-index/src/search.rs
    - crates/ctxhelm-index/src/symbols.rs
    - crates/ctxhelm-index/src/related_tests.rs
    - crates/ctxhelm-index/src/dependencies.rs
    - crates/ctxhelm-index/src/git.rs
    - crates/ctxhelm-index/src/traces.rs
  modified:
    - crates/ctxhelm-index/src/lib.rs

key-decisions:
  - "Kept ctxhelm-index crate root as the public facade and re-exported existing API names from focused private modules."
  - "Kept existing co-located tests in lib.rs and exposed only test-needed git parsing helpers as crate-visible internals."

patterns-established:
  - "Index concern modules own implementation; lib.rs owns module declarations and public re-exports."
  - "Shared inventory helpers remain crate-visible for sibling modules without widening public API."

requirements-completed: [CONT-04]

duration: 9m7s
completed: 2026-05-13
---

# Phase 01 Plan 03: Split ctxhelm-index Summary

**ctxhelm-index now uses focused concern modules while preserving crate-root public imports for CLI, compiler, and MCP consumers.**

## Performance

- **Duration:** 9m7s
- **Started:** 2026-05-13T12:13:28Z
- **Completed:** 2026-05-13T12:22:35Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Split inventory and lexical search out of the large `ctxhelm-index/src/lib.rs` into `inventory.rs` and `search.rs`.
- Split symbols, related test inference, and local dependency graph logic into `symbols.rs`, `related_tests.rs`, and `dependencies.rs`.
- Split git-backed co-change/current-diff/history logic and eval trace persistence into `git.rs` and `traces.rs`.
- Preserved crate-root public exports used by `ctxhelm-compiler`, `ctxhelm-mcp`, and the CLI.

## Task Commits

1. **Task 1: Create index facade and inventory/search modules** - `370e51d` (refactor)
2. **Task 2: Split symbols, related tests, and dependencies modules** - `2bdea3a` (refactor)
3. **Task 3: Split git/history/current-diff and trace modules, then run guardrails** - `8ea478f` (refactor)

## Files Created/Modified

- `crates/ctxhelm-index/src/lib.rs` - Crate-root facade with module declarations and public re-exports.
- `crates/ctxhelm-index/src/inventory.rs` - Inventory contracts, persistence, repo IDs, hashes, and path classification helpers.
- `crates/ctxhelm-index/src/search.rs` - Lexical search options, results, query tokenization, and scoring.
- `crates/ctxhelm-index/src/symbols.rs` - Symbol extraction and symbol search.
- `crates/ctxhelm-index/src/related_tests.rs` - Related-test inference, test map, and package-aware test command selection.
- `crates/ctxhelm-index/src/dependencies.rs` - Safe local import/dependency edge extraction.
- `crates/ctxhelm-index/src/git.rs` - Co-change hints, current-diff summaries, historical commit samples, and git helpers.
- `crates/ctxhelm-index/src/traces.rs` - Source-free eval trace append/list persistence.

## Decisions Made

- Kept `ctxhelm-index` import-compatible by re-exporting all existing public types and functions from `lib.rs`.
- Kept shared inventory helpers `pub(crate)` rather than public, matching the module-boundary goal without expanding the API.
- Kept tests co-located in `lib.rs` to avoid mixing structural refactor with test relocation.

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None. The only placeholder-related match is the existing `is_placeholder_test_script` helper name and implementation for filtering package-manager default test scripts.

## Issues Encountered

- Mechanical extraction initially exposed missing helper visibility and import boundaries during compilation. These were resolved inside the relevant task before committing.
- Self-check discovered pre-existing invalid local git refs (`master 2`, `master 3`) that make `git log --all` fail. Commit objects were verified directly with `git cat-file -e`, and the ref issue is logged in `deferred-items.md`.

## Verification

- `cargo test -p ctxhelm-index inventory -- --nocapture`
- `cargo test -p ctxhelm-index lexical_search -- --nocapture`
- `cargo test -p ctxhelm-index symbol -- --nocapture`
- `cargo test -p ctxhelm-index related_tests -- --nocapture`
- `cargo test -p ctxhelm-index dependency -- --nocapture`
- `cargo test -p ctxhelm-index current_diff -- --nocapture`
- `cargo test -p ctxhelm-index historical_commit -- --nocapture`
- `cargo test -p ctxhelm-index eval_traces -- --nocapture`
- `cargo test -p ctxhelm-index`
- `cargo test -p ctxhelm-compiler`
- `cargo test -p ctxhelm-mcp`
- `cargo test -p ctxhelm --test cli_compat`
- `cargo test --workspace`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 04 can split `ctxhelm-compiler` and `ctxhelm-mcp` behind stable facades with the existing compatibility guardrails still passing.

## Self-Check: PASSED

- Created files exist on disk.
- Task commits `370e51d`, `2bdea3a`, and `8ea478f` exist as commit objects.
- Note: `git log --all` is blocked by pre-existing invalid duplicate local refs; direct commit object verification passed.

---
*Phase: 01-compatibility-guardrails-module-boundaries*
*Completed: 2026-05-13*
