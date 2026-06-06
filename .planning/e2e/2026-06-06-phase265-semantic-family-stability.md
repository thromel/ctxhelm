# Phase 265 - Semantic Query-Family Stability

## Scope

Broaden the Phase 264 semantic query-family proof from `limit 10` to `limit 20`
on the four proof-fixture repositories, and make the semantic contribution
report distinguish repeated clean family signal from one-off aggregate path
counts.

## Implementation

- Added commit-level stability counters to
  `semanticContribution.queryFamilyContributions`:
  - `commitsWithSemanticOnlyTargetHits`
  - `commitsWithSemanticOnlyNonTargets`
  - `targetOnlyCommitCount`
  - `mixedCommitCount`
  - `noiseOnlyCommitCount`
  - `targetOnlyCommitRate`
  - `mixedCommitRate`
  - `noiseOnlyCommitRate`
- Added `semantic_query_family_unstable_hold` for families that have at least
  one clean semantic target-only commit but also mixed/noise commits in the same
  gate.
- Updated Markdown gate rendering and benchmarking docs with the new stability
  fields.

## Commands

```bash
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase265-semantic-stability-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase265-semantic-stability-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase265-semantic-stability-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase265-semantic-stability-verischema.json
```

## Evidence

| Repo | Decision | Default Recall@10 | Semantic Recall@10 | Routed Recall@10 | Stability finding |
| --- | --- | ---: | ---: | ---: | --- |
| RefactoringMiner | `hold` | `0.41857147` | `0.4285714` | `0.44357148` | `symbol_identifier` has `2` semantic-only target-hit commits, but also `5` non-target commits, `1` mixed commit, and `4` noise-only commits. |
| ctxhelm | `hold` | `0.44620585` | `0.44620585` | `0.44620585` | Phase 264 `broad_scope` candidate became unstable: `1` clean target-only commit and `1` noise-only commit. `domain_phrase` also has `1` clean target-only commit and `2` noise-only commits. |
| ReAgent | `hold` | `0.35` | `0.35` | `0.35` | Semantic remains pure noise by family: `commit_clue`, `symbol_identifier`, and `domain_phrase` all have non-target commits without semantic-only target-hit commits. |
| VeriSchema | `block` | `0.39382353` | `0.39382353` | `0.39382353` | Phase 264 `domain_phrase` candidate became unstable: `2` clean target-only commits and `1` noise-only commit. `broad_scope` has `2` clean target-only commits, `1` mixed commit, and `1` noise-only commit. |

## Result

The larger slice rejects runtime semantic query-family routing for the Phase 264
candidate families. The new counters show that the apparent `domain_phrase` and
`broad_scope` candidates are not stable enough: each has clean target-only
commits, but also same-family noise or mixed commits.

Semantic remains useful as an opt-in diagnostic and candidate-generation signal,
but Phase 265 moves the next R&D step away from routing semantic by coarse query
family alone. The next semantic experiments should target either stricter
within-family gating features or alternate local query/document construction,
then re-run the same stability counters before any runtime policy change.
