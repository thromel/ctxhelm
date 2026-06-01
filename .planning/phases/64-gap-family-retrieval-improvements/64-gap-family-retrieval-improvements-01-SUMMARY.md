---
phase: 64
title: Gap-Family Retrieval Improvements
status: completed
completed_at: 2026-05-30T06:51:42Z
requirements_addressed:
  - GAP-01
  - GAP-02
  - GAP-03
  - GAP-04
---

# Phase 64 Summary

## Completed

- Grouped measured Phase 63 gaps and selected the RefactoringMiner wrapper
  family as the first high-impact target.
- Implemented bounded lexical expansion for coding tasks.
- Added `lexical_expansion` as a distinct, non-protected retrieval signal so
  broader candidate generation does not make protected evidence impossible to
  satisfy under a 10-file budget.
- Preserved protected lexical hits before source expansion candidates.
- Added a source expansion reserve for source-free tasks without active anchors.
- Made protected-evidence eval budget-aware and kept test recall separate.

## Measured Outcome

RefactoringMiner:

- Recall@10: `0.1375 -> 0.7392`
- MRR@K: `0.1500 -> 0.7367`
- selected wrapper-family gap: `7 -> 1` missed files
- test Recall@10: unchanged at `0.0`
- protected miss rate: `0.0 -> 0.0526`

ctxhelm:

- Recall@10: `0.2049 -> 0.1947`
- MRR@K: `0.6333 -> 0.6083`
- protected miss rate: `0.2754 -> 0.2400`
- test Recall@10: unchanged at `0.0`

## Validation

```bash
cargo test -p ctxhelm-compiler -- --nocapture
cargo run -p ctxhelm -- eval benchmark --config .ctxhelm/e2e/phase62-default-config.json --format json
cargo test --workspace --no-fail-fast
cargo run -p ctxhelm -- --help
git diff --check
```

All validation commands passed.

## Follow-Up

- Phase 65 should decide whether this default is shippable given the small
  ctxhelm Recall@10 regression.
- Symbol protected evidence still has budget pressure when exact lexical and
  symbol candidates exceed the pack budget.
- The next retrieval-quality work should address test mapping for
  `src/test/java/org/refactoringminer/mcp/*.java` and ctxhelm documentation
  `no_candidate_signal` gaps.
