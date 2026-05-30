# Phase 67 E2E: Retrievable Target Eval Denominator

Date: 2026-05-30

## Objective

Separate historical eval's changed-file audit trail from the subset of files that could have been retrieved from the parent snapshot.

## Implementation

- Added `retrievalTargetFiles` to each historical eval commit row.
- Kept `safeChangedFiles` as the full source-free patch surface.
- Switched retrieval metrics and missing-file analysis to `retrievalTargetFiles`.
- Added a regression test for a commit that modifies an existing source file and adds a new docs file.

## Commands

```bash
cargo test -p ctxpack-compiler historical_eval_separates_new_files_from_retrievable_context_targets -- --nocapture
cargo test -p ctxpack-compiler ranking_metrics_historical_eval_reports_fixed_budget_without_source_text -- --nocapture
cargo run -p ctxpack -- \
  eval proof --config .ctxpack/e2e/phase62-default-config.json --format json
```

## Product-Proof Result

`ctxpack eval proof --config .ctxpack/e2e/phase62-default-config.json --format json`
produced `.ctxpack/e2e/phase67-retrievable-target-proof.json`.

```json
{
  "decision": "block",
  "defaultPromotionAllowed": false,
  "reason": "Blocked because default promotion requires every corpus to beat lexical; failing corpora: RefactoringMiner:Trail, ctxpack:Match."
}
```

Corpus verdicts:

| Corpus | Variant | Status | ctxpack Recall@10 | Lexical Recall@10 | Delta | Test Recall@10 | Protected miss-rate@10 |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `ctxpack_default` | `trail` | 0.7392 | 0.7792 | -0.0400 | 1.0000 | 0.0526 |
| ctxpack | `ctxpack_default` | `match` | 0.2277 | 0.2326 | -0.0049 | 1.0000 | 0.1500 |

## Decision

Phase 67 improves eval fidelity and removes false misses for unretrievable newly-added files. It does not make ctxpack production-ready. The release proof still blocks default promotion because every corpus does not beat lexical.

## Follow-Up

- Improve parser/precision ranking for source dependency gaps.
- Add better planning/docs/script candidate coverage for existing project artifacts.
- Reduce protected evidence miss-rate before promotion.
