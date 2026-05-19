---
phase: 58
title: Query Construction And Hybrid Fusion Controls
date: 2026-05-20
status: patterns
---

# Phase 58 Patterns

## Existing Patterns To Reuse

- `prepare_context_plan_with_paths_history_and_semantic` already centralizes planner inputs and should remain the main orchestration surface.
- `RankingInput` and candidate feature rows are the right place to add query facet provenance without inventing a parallel ranking path.
- Eval options in `crates/ctxpack-compiler/src/eval.rs` already support semantic toggles and should be extended rather than replaced.
- CLI JSON reports should remain backward compatible by adding optional fields.

## Closest Code Analogs

- `crates/ctxpack-compiler/src/planning.rs`: task parsing and retrieval orchestration.
- `crates/ctxpack-compiler/src/ranking.rs`: fusion features, signal kinds, and candidate evidence.
- `crates/ctxpack-compiler/src/eval.rs`: historical eval and benchmark variants.
- `crates/ctxpack-core/src/contracts.rs`: shared report contracts and feature rows.

## Implementation Notes

- Add query contracts in core before wiring compiler logic.
- Keep a plain task-text fallback so existing callers do not break.
- Make query traces source-free and bounded.
- Add unit tests for facet extraction before modifying rank fusion behavior.
