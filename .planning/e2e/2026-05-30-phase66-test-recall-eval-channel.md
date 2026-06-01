# Phase 66 E2E: Test Recall Evaluation Channel

Date: 2026-05-30

## Objective

Verify whether the v2.5 zero Test Recall@10 result was a real test-mapping failure or an evaluation-channel mismatch.

## Investigation

The Phase 65 proof showed `testRecallAt10 = 0.0` on both corpora. Inspection of per-commit output showed that `recommendedTests` already contained the changed test files, while `recommendedContextFiles` filled its 10-slot budget with target files before tests could appear.

A blunt test-slot reservation was tested first. It improved Test Recall@10, but degraded file recall and protected evidence coverage by displacing source files. That was rejected as the production default.

## Implementation

- Historical eval now computes test hits from `recommended_tests`.
- Target-file ranking remains source-focused.
- Related-test ranking now sorts by raw score before capped confidence.
- Exact test-name matches receive a small source-seed priority bonus.

## Commands

```bash
cargo test -p ctxhelm-compiler context_ranking_keeps_validation_tests_inside_budget -- --nocapture
cargo test -p ctxhelm-index related_tests_prefers_exact_tests_for_higher_ranked_source_seeds -- --nocapture
cargo run -p ctxhelm -- \
  eval proof --config .ctxhelm/e2e/phase62-default-config.json --format json
```

## Product-Proof Result

`ctxhelm eval proof --config .ctxhelm/e2e/phase62-default-config.json --format json`
produced `.ctxhelm/e2e/phase66-test-recall-proof.json`.

```json
{
  "decision": "block",
  "defaultPromotionAllowed": false,
  "reason": "Blocked because default promotion requires every corpus to beat lexical; failing corpora: RefactoringMiner:Trail, ctxhelm:Match."
}
```

Corpus verdicts:

| Corpus | Variant | Status | ctxhelm Recall@10 | Lexical Recall@10 | Delta | Test Recall@10 | Protected miss-rate@10 |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `ctxhelm_default` | `trail` | 0.7392 | 0.7792 | -0.0400 | 1.0000 | 0.0526 |
| ctxhelm | `ctxhelm_default` | `match` | 0.1879 | 0.1984 | -0.0105 | 1.0000 | 0.1700 |

## Decision

Phase 66 improves the product proof by correctly measuring validation-test recommendations. It does not make ctxhelm production-ready for default promotion. The default still trails lexical on RefactoringMiner and does not beat lexical on ctxhelm.

## Follow-Up

- Parser/precision work for repeated source dependency misses.
- Storage/indexing coverage for `.planning`, `docs`, and `scripts` gaps in the ctxhelm corpus.
- Protected evidence budget improvements for ctxhelm.
