---
phase: 58
title: Query Construction And Hybrid Fusion Controls
status: complete
completed_at: 2026-05-20
commit_scope: pending
requirements_addressed:
  - QUERY-01
  - QUERY-02
  - QUERY-03
  - QUERY-04
---

# Phase 58 Summary: Query Construction And Hybrid Fusion Controls

## What Changed

- Added source-free query facet, retriever query set, query trace, and fusion control contracts.
- Added optional `queryTrace` to `ContextPlan` and historical eval commit rows.
- Added compiler query construction for explicit paths, stack frames, symbols, error text, domain terms, commit clues, current diff anchors, and retriever-specific query sets.
- Promoted explicit task paths and stack-frame paths into anchor candidates.
- Preserved anchor dominance and exact-evidence protection through visible fusion controls.
- Fixed a regression where symbol matches for test files could be projected as target files instead of test candidates.
- Updated benchmarking docs with query trace debugging guidance.

## Privacy Boundary

Query traces store task hashes and bounded source-free facets. They do not store raw prompts, source snippets, or terminal output. The field formerly named `source` was deliberately renamed to `origin` so source-free guards do not treat query traces as source-bearing output.

## Verification

- `cargo test -p ctxhelm-core query_trace --no-fail-fast`
- `cargo test -p ctxhelm-compiler planning::tests --no-fail-fast`
- `cargo test -p ctxhelm-compiler prepare_context_plan_recommends_related_test_with_attribution_and_command --no-fail-fast`
- `cargo test -p ctxhelm --test cli_compat --no-fail-fast`
- `bash scripts/smoke-v23-eval.sh`
- `cargo test --workspace --no-fail-fast`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`

All listed verification passed.

## Notes

- Semantic retrieval remains opt-in.
- Query traces make semantic non-lift debuggable: if semantic phrases are weak or absent, semantic recall should not be expected to improve.
- Historical eval now carries per-commit query traces for source-free process debugging.
