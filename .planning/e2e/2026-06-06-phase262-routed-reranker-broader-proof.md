# Phase 262: Routed Reranker Broader Proof

## Goal

Broaden Phase 261's routed-reranker policy proof across the clean four-repo
fixture set and reject any route family that does not hold up outside ctxhelm.

Phase 261 routed both `commit_clue` and `symbol_identifier`. That was a
reasonable first candidate from ctxhelm-only evidence, but not enough for
promotion. This phase tests the policy on RefactoringMiner, ctxhelm, ReAgent,
and VeriSchema.

## Finding

The first broader run rejected `symbol_identifier` routing:

```text
ReAgent routedRerankerContribution.targetHitDelta: -5
ReAgent routedRerankerContribution.defaultOnlyTargetHitCount: 5
ReAgent routedRerankerContribution.regressedCommitCount: 2
```

The route policy was tightened to `commit_clue` only. `symbol_identifier`
continues to appear in `queryFamilyContributions`, but routed ranking now keeps
default ranking for that family.

## Change

- Narrow `query_family_routed_reranked` from `commit_clue` +
  `symbol_identifier` to `commit_clue` only.
- Update focused route-policy tests to assert that `symbol_identifier` is not
  currently routed.
- Preserve the eval-only status. This still does not enable runtime reranking.

## Proof

Artifacts:

- `.ctxhelm/e2e/phase262-routed-reranker-broader-proof-refactoringminer.json`
- `.ctxhelm/e2e/phase262-routed-reranker-broader-proof-ctxhelm.json`
- `.ctxhelm/e2e/phase262-routed-reranker-broader-proof-reagent.json`
- `.ctxhelm/e2e/phase262-routed-reranker-broader-proof-verischema.json`

Commands:

```bash
cargo test -p ctxhelm-compiler reranker --locked
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase262-routed-reranker-broader-proof-refactoringminer.json
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase262-routed-reranker-broader-proof-ctxhelm.json
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase262-routed-reranker-broader-proof-reagent.json
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase262-routed-reranker-broader-proof-verischema.json
```

| Repo | Default Recall@10 | Full Reranker Recall@10 | Routed Recall@10 | Routed target-hit delta | Routed default-only churn | Routed regressions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 0.5383333 | 0.5383333 | 0.5383333 | 0 | 0 | 0 |
| ctxhelm | 0.3536905 | 0.67595243 | 0.5005952 | +9 | 0 | 0 |
| ReAgent | 0.5 | 0.35 | 0.5 | 0 | 0 | 0 |
| VeriSchema | 0.48764706 | 0.34843138 | 0.48764706 | 0 | 0 | 0 |

Full reranking remains unsafe:

- ReAgent full reranker: `targetHitDelta = -5`,
  `defaultOnlyTargetHitCount = 5`.
- VeriSchema full reranker: `targetHitDelta = -7`,
  `defaultOnlyTargetHitCount = 10`.

## Decision

Promote the corrected routed policy as the next eval candidate, not runtime
behavior.

The broader proof shows the narrower `commit_clue` route removes all measured
routed regressions and all default-only churn across four repos while keeping a
smaller clean ctxhelm lift. That is good enough to continue provider-policy
opt-in design work, but not enough for unconditional reranker promotion.
