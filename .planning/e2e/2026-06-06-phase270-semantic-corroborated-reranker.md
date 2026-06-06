# Phase 270 - Semantic-Corroborated Reranker

## Scope

Test the Phase 269 fusion hypothesis with a source-free eval-only reranker.
Phase 269 showed that ctxhelm, ReAgent, and RefactoringMiner had semantic
candidate target hits that final top-K selection dropped. This phase asks
whether a bounded semantic bonus, applied only when another local signal
corroborates the candidate, can recover those targets safely.

This phase does not change runtime provider policy, default ranking, planner
behavior, or semantic documents.

## Implementation

Added the eval-only `semantic_corroborated_reranked` gate variant.

The variant:

- enables local semantic retrieval;
- keeps the same protected source floor used by the local metadata reranker;
- gives semantic candidates a bounded score bonus only when at least one
  non-semantic source-free signal also supports the candidate;
- reports named wins and regressions against the default ranking.

## Validation Commands

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler semantic_corroborated --locked
cargo test -p ctxhelm-compiler semantic_contribution --locked
cargo build -p ctxhelm --features local-embeddings
```

Four-repo gate refresh:

```bash
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase270-semantic-corroborated-reranker-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase270-semantic-corroborated-reranker-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase270-semantic-corroborated-reranker-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase270-semantic-corroborated-reranker-verischema.json
```

## Results

| Repo | Decision | Default Recall@10 | Local Semantic Recall@10 | Semantic-Corroborated Recall@10 | Named wins | Named regressions |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| ctxhelm | `hold` | `0.44620585` | `0.44620585` | `0.44333735` | `7` | `9` |
| ReAgent | `hold` | `0.35` | `0.35` | `0.325` | `2` | `3` |
| RefactoringMiner | `hold` | `0.41857147` | `0.4285714` | `0.5619047` | `5` | `0` |
| VeriSchema | `block` | `0.39382353` | `0.39382353` | `0.36960787` | `0` | `3` |

## Decision

Reject global semantic-corroborated reranking.

The rule is promising but not generally safe. It creates a strong
RefactoringMiner win with no named regressions, but it regresses ctxhelm,
ReAgent, and VeriSchema. The next semantic R&D should either identify a stable
query-family/corpus-shape route for the RefactoringMiner lift or reject this
fusion path if no separator survives broader proof.

Operational note: run `cargo build -p ctxhelm --features local-embeddings`
immediately before local-fastembed gates. A later featureless `cargo run -p
ctxhelm` can overwrite `./target/debug/ctxhelm` with a binary that reports
`local_fastembed` unavailable.
