# Phase 64 E2E: Gap-Family Retrieval Improvements

Date: 2026-05-30

## Objective

Convert a measured Phase 63 RefactoringMiner gap family into a targeted
retrieval improvement without enabling cloud providers or changing MCP tool
surface.

## Target Gap

Phase 63 selected:

- repo: `RefactoringMiner`
- gap: `no_candidate_signal`
- path family: `src/main/java/org/refactoringminer/astDiff/matchers/wrappers/*.java`
- missed count: `7`

The fix targeted candidate eligibility and budgeted selection for source files
that only become reachable after broader lexical evidence seeds graph expansion.

## Implementation

- Added a wider lexical candidate limit for coding tasks while keeping symbol
  and semantic limits bounded.
- Split lexical evidence into protected `lexical` hits and non-protected
  `lexical_expansion` hits.
- Preserved protected lexical hits before expansion candidates in final target
  selection.
- Added a source lexical-expansion reserve for source-free tasks with no active
  anchors.
- Made protected-evidence eval budget-aware so the metric does not require more
  protected candidates than the pack budget can hold.
- Excluded lexical test candidates from protected target-file evidence; test
  quality remains reported through `testRecallAt10`.

## Commands

```bash
cargo test -p ctxhelm-compiler -- --nocapture
cargo run -p ctxhelm -- \
  eval benchmark --config .ctxhelm/e2e/phase62-default-config.json --format json
cargo test --workspace --no-fail-fast
cargo run -p ctxhelm -- --help
git diff --check
```

Large JSON reports were kept under ignored `.ctxhelm/e2e/`.

## Results

| Repo | Phase 63 Recall@10 | Phase 64 Recall@10 | Delta | Phase 63 MRR@K | Phase 64 MRR@K | Test Recall@10 delta | Protected miss-rate delta | Runtime delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 0.1375 | 0.7392 | +0.6017 | 0.1500 | 0.7367 | +0.0000 | +0.0526 | +17.2s |
| ctxhelm | 0.2049 | 0.1947 | -0.0102 | 0.6333 | 0.6083 | +0.0000 | -0.0354 | +1.7s |

## Gap Outcome

The selected wrapper-family gap improved:

- before: `no_candidate_signal`, missed count `7`
- after: `ranked_below_budget_dependency`, missed count `1`

That means the family is now generally candidate-reachable and graph-related;
the remaining miss is a budget/precision issue rather than candidate absence.

## Residual Risk

The RefactoringMiner protected miss rate is no longer zero because strong symbol
candidates can still exceed the 10-file context budget after lexical protection.
Lexical protected evidence had zero misses in the final report, but symbol
budget pressure remains a follow-up for Phase 65 release proof or a future
symbol-budget pass.

The ctxhelm corpus had a small Recall@10 regression (`-0.0102`) while protected
miss rate improved. Phase 65 should decide whether this default is shippable,
needs a per-repo/task gate, or should stay behind an eval flag.

## Decision

Phase 64 is accepted as a targeted gap-family improvement, not a final product
release gate. The selected high-impact family improved substantially with
source-free proof and full validation passing. Remaining mixed-corpus trade-offs
move to Phase 65.
