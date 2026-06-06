# Phase 272 - Semantic-Corroborated Path-Family Diagnostics

## Scope

Phase 271 rejected query-family-only routing for
`semantic_corroborated_reranked`. This phase adds source-free path-family
contribution diagnostics to determine whether the RefactoringMiner lift is
separable by corpus/path shape.

This phase is diagnostic only. It does not change runtime provider policy,
default ranking, planner behavior, semantic documents, or MCP behavior.

## Implementation

Added `pathFamilyContributions` to `RerankerContributionSummary`. Each row
reports:

- path family;
- reranker-only target hits;
- default-only target hits;
- target-hit delta;
- route candidate, churn hold, neutral hold, or block recommendation;
- source-free example paths for both reranker-only and default-only movement.

The path family classifier uses only path metadata, not source text. Families
include `planning`, `docs`, `scripts`, `docker`, `config`, `java_source`,
`java_test`, `python_source`, `python_test`, `rust_source`, `rust_test`,
`javascript_source`, `test`, and `other`.

## Validation Commands

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler semantic_corroborated --locked
cargo test -p ctxhelm-compiler reranker_contribution --locked
cargo build -p ctxhelm --features local-embeddings
```

Four-repo gate refresh:

```bash
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase272-semantic-corroborated-path-family-diagnostics-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase272-semantic-corroborated-path-family-diagnostics-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase272-semantic-corroborated-path-family-diagnostics-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase272-semantic-corroborated-path-family-diagnostics-verischema.json
```

## Results

| Repo | Semantic-corroborated target delta | Useful path families | Blocked or churned path families |
| --- | ---: | --- | --- |
| ctxhelm | `-7` | `scripts +3` | `docs +11 but churn`, `planning -3 churn`, `rust_source -18 churn` |
| ReAgent | `-4` | none | `scripts -1 block`, `planning -3 churn` |
| RefactoringMiner | `+5` | `docs +2`, `config +1`, `java_source +1`, `java_test +1` | none |
| VeriSchema | `-8` | `scripts +2` | `python_source -10 block` |

## Decision

Reject broad path-family routing for `semantic_corroborated_reranked`.

Path families explain why RefactoringMiner benefits: the lift is concentrated
in docs, Gradle config, and Java MCP source/test files with no default-only
target churn. But that does not generalize:

- ctxhelm has positive docs movement but churns default docs/planning/rust
  targets badly;
- ReAgent loses planning and script targets;
- VeriSchema gains scripts but loses Python source targets, which are the core
  implementation evidence for that corpus.

The remaining semantic path is narrower than "query family" or "path family":
future work should test corpus-shape constraints such as Java/MCP/package
coherence, or shift effort to VeriSchema query/document construction where the
current semantic provider fails to recover Python implementation targets.
