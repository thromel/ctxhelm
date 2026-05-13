---
phase: 01-compatibility-guardrails-module-boundaries
plan: 04
subsystem: architecture
tags: [rust, module-boundaries, ctxpack-compiler, ctxpack-mcp, compatibility]

# Dependency graph
requires:
  - phase: 01-compatibility-guardrails-module-boundaries
    provides: "CLI, JSON contract, MCP, and ctxpack-index compatibility guardrails from Plans 01-03"
provides:
  - "ctxpack-compiler split into planning, packs, cards, and eval modules behind stable crate-root exports"
  - "ctxpack-mcp split into protocol, schemas, tools, resources, and prompts modules behind stable crate-root exports"
  - "Full compatibility validation after staged compiler and MCP splits"
affects: [ctxpack-compiler, ctxpack-mcp, cli, mcp, public-api]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Crate-root facade with focused private modules"
    - "Crate-visible helpers only where module boundaries require shared internals"

key-files:
  created:
    - crates/ctxpack-compiler/src/planning.rs
    - crates/ctxpack-compiler/src/packs.rs
    - crates/ctxpack-compiler/src/cards.rs
    - crates/ctxpack-compiler/src/eval.rs
    - crates/ctxpack-mcp/src/protocol.rs
    - crates/ctxpack-mcp/src/schemas.rs
    - crates/ctxpack-mcp/src/tools.rs
    - crates/ctxpack-mcp/src/resources.rs
    - crates/ctxpack-mcp/src/prompts.rs
  modified:
    - crates/ctxpack-compiler/src/lib.rs
    - crates/ctxpack-mcp/src/lib.rs

key-decisions:
  - "Kept ctxpack-compiler and ctxpack-mcp crate roots as stable public facades while moving implementation into concern-focused modules."
  - "Used pub(crate) only for shared internals needed across the new module boundaries and existing crate-local tests."

patterns-established:
  - "Compiler facade re-exports public planning, pack, card, render, trace, and historical-eval APIs."
  - "MCP facade re-exports only run_server and run_stdio_server while keeping protocol implementation private."

requirements-completed: [CONT-04]

# Metrics
duration: 8m10s
completed: 2026-05-13
---

# Phase 01 Plan 04: Compiler and MCP Module Boundary Summary

**Compiler and MCP implementation split into focused Rust modules while preserving crate-root APIs and compatibility guardrails.**

## Performance

- **Duration:** 8m10s
- **Started:** 2026-05-13T12:26:48Z
- **Completed:** 2026-05-13T12:34:58Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Split `ctxpack-compiler` into `planning`, `packs`, `cards`, and `eval` modules with stable public exports from `ctxpack_compiler`.
- Split `ctxpack-mcp` into `protocol`, `schemas`, `tools`, `resources`, and `prompts` modules with stable `run_stdio_server` and `run_server` exports.
- Preserved public CLI, JSON, compiler, and MCP behavior under focused stage checks and full workspace validation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Split compiler planning, packs, cards, and eval modules** - `bb7c987` (refactor)
2. **Task 2: Split MCP protocol, schema, tool, resource, and prompt modules** - `19d02ae` (refactor)
3. **Task 3: Run full compatibility validation after module splits** - `2dbf56e` (test, empty validation commit)

## Files Created/Modified

- `crates/ctxpack-compiler/src/lib.rs` - Stable compiler facade and existing crate-local tests.
- `crates/ctxpack-compiler/src/planning.rs` - Context planning, ranking, anchors, confidence, and low-information helpers.
- `crates/ctxpack-compiler/src/packs.rs` - Context pack compilation, Markdown rendering, snippets, and checklist rendering.
- `crates/ctxpack-compiler/src/cards.rs` - Source-free context card generation and rendering helpers.
- `crates/ctxpack-compiler/src/eval.rs` - Historical eval reports, eval traces, parent snapshot extraction, and metrics.
- `crates/ctxpack-mcp/src/lib.rs` - Stable MCP facade, tool-name constants, and compatibility tests.
- `crates/ctxpack-mcp/src/protocol.rs` - JSON-RPC protocol loop, request dispatch, and error responses.
- `crates/ctxpack-mcp/src/schemas.rs` - Initialize, tools/list, resources/list, and prompts/list schema payloads.
- `crates/ctxpack-mcp/src/tools.rs` - MCP tool handlers and tool argument contracts.
- `crates/ctxpack-mcp/src/resources.rs` - MCP resource reads, repo/file/symbol resources, pack guide, and session pack cache.
- `crates/ctxpack-mcp/src/prompts.rs` - MCP prompt handlers and workflow prompt rendering.

## Decisions Made

- Kept module splits internal to the owning crates; public compiler and MCP call sites remain unchanged.
- Kept the MCP pack cache session-scoped as characterized by existing compatibility tests.
- Used an empty Task 3 commit to preserve one atomic commit per plan task for a validation-only task.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Restored module-local derives, imports, and crate-visible helpers after mechanical extraction**
- **Found during:** Tasks 1 and 2
- **Issue:** Initial mechanical splits separated some `derive` attributes from their structs and left a few helpers/imports with visibility that no longer matched the new module boundaries.
- **Fix:** Reattached derive/serde attributes, added precise imports, and exposed only needed crate-visible internals for cross-module calls and existing tests.
- **Files modified:** `crates/ctxpack-compiler/src/*.rs`, `crates/ctxpack-mcp/src/*.rs`
- **Verification:** `cargo test -p ctxpack-compiler`, `cargo test -p ctxpack-mcp`, and final workspace validation passed.
- **Committed in:** `bb7c987`, `19d02ae`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** No public behavior drift; fixes were limited to making the planned internal module split compile and test cleanly.

## Issues Encountered

- Final explicit validation commands briefly waited on Cargo build locks because they were launched concurrently; all completed successfully without code changes.

## Known Stubs

None. Stub scan found only the intentional MCP session-cache error text: `pack resource is not available in this MCP session; call prepare_task first`.

## Verification

- `cargo test -p ctxpack-compiler` passed.
- `cargo test -p ctxpack-core public_json_shape -- --nocapture` passed.
- `cargo test -p ctxpack-compiler public_json_shape -- --nocapture` passed.
- `cargo test -p ctxpack-mcp` passed.
- `cargo test -p ctxpack --test cli_compat serve_mcp -- --nocapture` passed.
- `cargo test --workspace` passed.
- `cargo run -p ctxpack -- --help` passed and listed `serve-mcp`.
- `cargo test -p ctxpack --test cli_compat` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 01 is complete. The workspace now has compatibility guardrails plus stable module boundaries for CLI, JSON contracts, index, compiler, and MCP, ready for later freshness/privacy/retrieval-quality phases.

## Self-Check: PASSED

- Verified SUMMARY and all created module files exist.
- Verified task commits `bb7c987`, `19d02ae`, and `2dbf56e` exist via `git show -s`.
- Note: the prescribed `git log --all` lookup is blocked in this repository by an existing malformed ref `refs/heads/master 2`; direct commit lookup succeeded.

---
*Phase: 01-compatibility-guardrails-module-boundaries*
*Completed: 2026-05-13*
