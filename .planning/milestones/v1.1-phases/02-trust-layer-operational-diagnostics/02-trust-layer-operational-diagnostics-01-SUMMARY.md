---
phase: 02-trust-layer-operational-diagnostics
plan: 01
subsystem: trust-layer
tags: [rust, serde-json, diagnostics, privacy-policy, source-read-policy]

requires:
  - phase: 01-compatibility-guardrails-module-boundaries
    provides: Public JSON shape guardrails and ctxhelm-index module facade from Phase 1
provides:
  - Additive source-free diagnostics contracts on ContextPlan and ContextPack
  - Minimal cache and trace status contracts for later Phase 2 wiring
  - Central ctxhelm-index policy module for privacy classification and safe source reads
  - Typed source-read skip reasons for policy, binary, non-UTF-8, oversized, and unreadable cases
affects: [ctxhelm-core, ctxhelm-index, ctxhelm-compiler, trust-layer-operational-diagnostics]

tech-stack:
  added: []
  patterns:
    - serde defaulted additive fields for backward-compatible contracts
    - source-free Diagnostic values with stable reason codes
    - ctxhelm-index crate-root facade re-exporting central policy contracts

key-files:
  created:
    - crates/ctxhelm-index/src/policy.rs
    - .planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-01-SUMMARY.md
  modified:
    - crates/ctxhelm-core/src/contracts.rs
    - crates/ctxhelm-compiler/src/planning.rs
    - crates/ctxhelm-compiler/src/packs.rs
    - crates/ctxhelm-index/src/lib.rs
    - crates/ctxhelm-index/src/inventory.rs

key-decisions:
  - "Diagnostics are additive serde fields on ContextPlan and ContextPack, with riskFlags preserved unchanged for compatibility."
  - "ctxhelm-index policy.rs owns path role classification and safe source-read outcomes, while lib.rs remains the public facade."
  - "Safe source reads return typed SourceRead values with source-free diagnostics instead of CLI/MCP formatting strings."

patterns-established:
  - "Use Diagnostic plus DiagnosticSeverity for machine-readable trust-layer warnings and errors."
  - "Represent source-read skips with SourceReadStatus::Skipped(SourceReadReason) and stable diagnostic codes."
  - "Delegate inventory file role classification to policy.rs instead of keeping path helpers in inventory.rs."

requirements-completed: [SAFE-03, DIAG-01, DIAG-02]

duration: 6m52s
completed: 2026-05-13
---

# Phase 02 Plan 01: Trust Layer Operational Diagnostics Summary

**Source-free diagnostics contracts and a central privacy/source-read policy for ctxhelm trust-layer wiring.**

## Performance

- **Duration:** 6m52s
- **Started:** 2026-05-13T13:10:41Z
- **Completed:** 2026-05-13T13:17:33Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added `Diagnostic`, `DiagnosticSeverity`, `CacheStatus`, and `TraceStatus` contracts in `ctxhelm-core`.
- Added defaulted `diagnostics` arrays to `ContextPlan` and `ContextPack` while preserving existing `riskFlags` and `warnings` behavior.
- Created `ctxhelm-index/src/policy.rs` with central path classification, typed `SourceRead` results, and stable diagnostic codes for source-read skips.
- Delegated inventory role classification through the new policy module and re-exported the public policy API from `ctxhelm-index`.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Add failing diagnostics contract shape tests** - `cc22c72` (test)
2. **Task 1 GREEN: Add additive diagnostics contracts** - `47f6e50` (feat)
3. **Task 2 RED: Add failing central policy tests** - `3c7b7de` (test)
4. **Task 2 GREEN: Centralize source-read policy** - `f9d7ae0` (feat)

## Files Created/Modified

- `crates/ctxhelm-core/src/contracts.rs` - Added diagnostics/cache/trace contracts, additive `diagnostics` fields, and public JSON compatibility tests.
- `crates/ctxhelm-compiler/src/planning.rs` - Initializes new plan diagnostics field in base plans.
- `crates/ctxhelm-compiler/src/packs.rs` - Carries plan diagnostics into compiled packs.
- `crates/ctxhelm-index/src/policy.rs` - New central privacy/source-read policy module with table-driven tests.
- `crates/ctxhelm-index/src/inventory.rs` - Delegates path role classification and language detection to policy.rs.
- `crates/ctxhelm-index/src/lib.rs` - Re-exports the minimal policy API from the crate-root facade.

## Decisions Made

- Kept `riskFlags` unchanged and added `diagnostics` as a defaulted field so older serialized plans/packs still deserialize.
- Used source-free diagnostics with code, severity, message, paths, and count only; no source snippets or task/prompt text are stored in diagnostics.
- Kept source-read policy typed at the Rust API boundary with `SourceReadStatus` and `SourceReadReason`, leaving CLI/MCP rendering for later Phase 2 plans.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated compiler construction sites for additive contract fields**
- **Found during:** Task 1 (Add additive diagnostics contracts)
- **Issue:** Adding public `diagnostics` fields to `ContextPlan` and `ContextPack` required existing compiler struct initializers to populate the new fields before downstream crates could compile.
- **Fix:** Initialized empty diagnostics in base plans and propagated plan diagnostics into compiled packs.
- **Files modified:** `crates/ctxhelm-compiler/src/planning.rs`, `crates/ctxhelm-compiler/src/packs.rs`
- **Verification:** `cargo test -p ctxhelm-core -p ctxhelm-index` and `cargo test --workspace` passed.
- **Committed in:** `47f6e50`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required to preserve compile compatibility after additive public contract changes. No retrieval ranking, cloud, or autonomous-editing scope was introduced.

## Issues Encountered

None.

## Authentication Gates

None.

## Known Stubs

None.

## Verification

- `cargo test -p ctxhelm-core public_json_shape -- --nocapture` passed.
- `cargo test -p ctxhelm-index policy -- --nocapture` passed.
- `cargo test -p ctxhelm-core -p ctxhelm-index` passed.
- `cargo test --workspace` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 02 can wire diagnostics into inventory freshness and cache-write behavior using the additive core contracts and central policy API added here.

## Self-Check: PASSED

- Found files: `crates/ctxhelm-index/src/policy.rs` and this SUMMARY.
- Found task commits via direct commit-object checks: `cc22c72`, `47f6e50`, `3c7b7de`, `f9d7ae0`.
- Note: `git log --oneline --all` is still blocked by the pre-existing malformed local ref `refs/heads/master 2`; direct `git cat-file -e <hash>^{commit}` checks confirmed all Plan 02-01 commits exist.

---
*Phase: 02-trust-layer-operational-diagnostics*
*Completed: 2026-05-13*
