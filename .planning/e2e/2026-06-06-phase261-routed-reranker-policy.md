# Phase 261: Routed Reranker Policy

## Goal

Turn Phase 260's query-family routing evidence into an executable eval-only
policy and measure whether it keeps useful reranker lift while removing target
churn.

The full local metadata reranker can improve target recall on ctxhelm, but the
aggregate variant still churns default target hits. Phase 260 showed that
`commit_clue` and `symbol_identifier` families had clean lift while
`domain_phrase` needed to remain held. Phase 262 later broadened this proof and
rejected `symbol_identifier` routing after ReAgent regression, leaving
`commit_clue` as the only current routed family.

## Change

- Add `query_family_routed_reranked` to the semantic/precision gate variants.
- Add `routedRerankerContribution` to compare that routed variant against
  `ctxhelm_default`.
- Apply the local metadata reranker only for the initial conservative
  route-safe families: `commit_clue` and `symbol_identifier`.
- Keep default ranking for held families such as `domain_phrase`,
  `explicit_path`, `broad_scope`, and low-information tasks.
- Add routed diagnostics:
  - `routed_reranker_clean_lift`
  - `routed_reranker_churn_hold`
  - `routed_reranker_regression`
  - `routed_reranker_neutral`

This is still eval-only. It does not enable routed reranking in runtime
provider policy.

## Proof

Artifacts:

- `.ctxhelm/e2e/phase261-routed-reranker-policy-refactoringminer.json`
- `.ctxhelm/e2e/phase261-routed-reranker-policy-ctxhelm.json`

Commands:

```bash
cargo test -p ctxhelm-compiler reranker --locked
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase261-routed-reranker-policy-refactoringminer.json
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase261-routed-reranker-policy-ctxhelm.json
```

RefactoringMiner:

```text
decision: hold
namedRegressions: []
ctxhelm_default Recall@10: 0.5383333
local_metadata_reranked Recall@10: 0.5383333
query_family_routed_reranked Recall@10: 0.5383333
routedRerankerContribution.targetHitDelta: 0
routedRerankerContribution.defaultOnlyTargetHitCount: 0
diagnostic: routed_reranker_neutral
```

ctxhelm:

```text
decision: hold
namedRegressions: []
ctxhelm_default Recall@10: 0.3036905
local_metadata_reranked Recall@10: 0.6342858
query_family_routed_reranked Recall@10: 0.48845237
rerankerContribution.targetHitDelta: +18
rerankerContribution.defaultOnlyTargetHitCount: 2
routedRerankerContribution.targetHitDelta: +11
routedRerankerContribution.defaultOnlyTargetHitCount: 0
routedRerankerContribution.improvedCommitCount: 5
routedRerankerContribution.regressedCommitCount: 0
diagnostic: routed_reranker_clean_lift
```

## Decision

Promote the routed reranker policy as an eval candidate, not a runtime default.

The routed policy gives up some full-reranker lift in exchange for removing all
measured default-only target churn on ctxhelm while staying neutral on
RefactoringMiner. The next R&D step is broader routed-policy proof across more
corpora, followed by an explicit provider-policy opt-in decision if the
zero-churn lift repeats.
