# Phase 260: Query-Family Reranker Routing Diagnostics

## Goal

Move local metadata reranker R&D from aggregate lift/churn numbers toward
routeable, source-free query-family evidence.

Phases 257-259 made the reranker safe enough to evaluate and safe enough to
opt in at runtime, but Phase 258/259 still showed corpus-specific behavior and
target churn. The missing evidence was where the reranker helps, where it is
neutral, and where it must remain held because it replaces target files that
default retrieval already found.

## Change

- Add `rerankerContribution.queryFamilyContributions` to semantic/precision
  gate reports.
- Classify each measured commit into a primary source-free query family:
  `explicit_path`, `stack_or_error`, `symbol_identifier`, `commit_clue`,
  `domain_phrase`, `broad_scope`, `low_information`, or a task-type fallback.
- Report per-family evaluated commits, improved/regressed/neutral counts,
  target-hit deltas, reranker-only target hits, default-only target hits,
  example cases, and a `routingRecommendation`.
- Emit source-free diagnostics for:
  - `reranker_query_family_route_candidate`
  - `reranker_query_family_churn_hold`
- Keep reranking default-off. This phase adds routing evidence, not runtime
  promotion.

## Proof

Artifacts:

- `.ctxhelm/e2e/phase260-query-family-reranker-routing-refactoringminer.json`
- `.ctxhelm/e2e/phase260-query-family-reranker-routing-ctxhelm.json`

Commands:

```bash
cargo test -p ctxhelm-compiler reranker_contribution --locked
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase260-query-family-reranker-routing-refactoringminer.json
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase260-query-family-reranker-routing-ctxhelm.json
```

RefactoringMiner:

```text
decision: hold
namedRegressions: []
rerankerContribution.targetHitDelta: 0
queryFamilyContributions:
  domain_phrase: evaluated=2, targetHitDelta=0, routingRecommendation=hold_neutral
  symbol_identifier: evaluated=8, targetHitDelta=0, routingRecommendation=hold_neutral
diagnostic: reranker_contribution_neutral
```

ctxhelm:

```text
decision: hold
namedRegressions: []
rerankerContribution.targetHitDelta: +15
rerankerContribution.improvedCommitCount: 8
rerankerContribution.regressedCommitCount: 0
rerankerContribution.defaultOnlyTargetHitCount: 2
queryFamilyContributions:
  domain_phrase: evaluated=5, targetHitDelta=+7, defaultOnlyTargetHitCount=2, routingRecommendation=hold_churn
  commit_clue: evaluated=2, targetHitDelta=+4, defaultOnlyTargetHitCount=0, routingRecommendation=route_candidate
  symbol_identifier: evaluated=2, targetHitDelta=+4, defaultOnlyTargetHitCount=0, routingRecommendation=route_candidate
  broad_scope: evaluated=1, targetHitDelta=0, routingRecommendation=hold_neutral
diagnostics: reranker_contribution_target_lift, reranker_contribution_protected_target_improved, reranker_contribution_target_churn, reranker_query_family_route_candidate, reranker_query_family_churn_hold
```

## Decision

Promote query-family diagnostics, not default reranking.

The new evidence is more actionable than the aggregate Phase 258/259 result:
RefactoringMiner stays neutral by family, while ctxhelm has two clean route
candidates and one churn-held winning family. The next R&D step is to evaluate
a gated route or learned fusion policy that can enable the reranker only for
families that repeatedly show lift without default-only target churn.
