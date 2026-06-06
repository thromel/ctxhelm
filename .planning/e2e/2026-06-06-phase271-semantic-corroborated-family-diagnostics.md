# Phase 271 - Semantic-Corroborated Family Diagnostics

## Scope

Phase 270 showed that `semantic_corroborated_reranked` creates a large
RefactoringMiner lift but regresses ctxhelm, ReAgent, and VeriSchema. This
phase adds source-free query-family contribution diagnostics for that variant
and reruns the same four-repo gate to test whether the RefactoringMiner lift has
a stable route.

This phase is diagnostic only. It does not change runtime provider policy,
default ranking, planner behavior, semantic documents, or MCP behavior.

## Implementation

Added `semanticCorroboratedRerankerContribution` to the semantic/precision gate
report. It mirrors the existing reranker contribution contract:

- improved, regressed, and neutral commit counts;
- target-hit delta;
- reranker-only and default-only target hit counts;
- query-family contribution rows;
- route candidate, churn hold, neutral hold, and regression block
  recommendations;
- named source-free example cases with the correct
  `semantic_corroborated_reranked` variant label.

## Validation Commands

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler semantic_corroborated --locked
cargo test -p ctxhelm-compiler reranker_contribution --locked
cargo build -p ctxhelm --features local-embeddings
```

Four-repo gate refresh:

```bash
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase271-semantic-corroborated-family-diagnostics-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase271-semantic-corroborated-family-diagnostics-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase271-semantic-corroborated-family-diagnostics-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase271-semantic-corroborated-family-diagnostics-verischema.json
```

## Results

| Repo | Default Recall@10 | Semantic-Corroborated Recall@10 | Target-hit delta | Improved | Regressed | Default-only targets |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| ctxhelm | `0.44620585` | `0.44333735` | `-7` | `7` | `9` | `37` |
| ReAgent | `0.35` | `0.325` | `-4` | `2` | `3` | `6` |
| RefactoringMiner | `0.41857147` | `0.5619047` | `+5` | `5` | `0` | `0` |
| VeriSchema | `0.39382353` | `0.36960787` | `-8` | `0` | `3` | `10` |

Query-family contribution:

| Query family | RefactoringMiner | ctxhelm | ReAgent | VeriSchema | Decision |
| --- | --- | --- | --- | --- | --- |
| `domain_phrase` | route candidate, `+2`, no churn | block regression, `+5` net but `7` default-only targets | block regression, `+1` net but `1` default-only target | block regression, `-1` | Reject cross-repo route |
| `symbol_identifier` | route candidate, `+2`, no churn | hold churn, `+5` net but `1` default-only target | block regression, `-5` | hold neutral | Reject cross-repo route |
| `commit_clue` | route candidate, `+1`, no churn | not present | hold neutral | not present | Insufficient route evidence |
| `broad_scope` | hold neutral | block regression, `-17` | not present | block regression, `-7` | Block |

## Decision

Reject query-family-only routing for `semantic_corroborated_reranked`.

The RefactoringMiner lift is real, clean, and source-free, but the same coarse
query families are unsafe on the other measured repos. The next semantic R&D
should avoid another route based only on query family. Viable next experiments
are:

1. corpus-shape diagnostics that explain why RefactoringMiner is separable;
2. path-role or package-shape constraints for Java/MCP/documentation changes;
3. separate query/document construction work for VeriSchema, where semantic
   candidate generation still fails for important Python evaluation paths.
