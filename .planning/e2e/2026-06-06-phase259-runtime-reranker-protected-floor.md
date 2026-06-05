# Phase 259: Runtime Reranker Protected Floor

## Goal

Align the policy-enabled runtime local metadata reranker with the protected
evidence safety contract proven in eval.

Phase 257 made `local_metadata_reranked` safe inside historical eval by
preserving protected source evidence and validation-test reserves. Phase 258
made reranker contribution inspectable. This phase closes the runtime gap:
`prepare_task` previously applied a policy-enabled local metadata reranker as a
full reorder.

## Change

- Change runtime `rerank_with_local_metadata` to use a protected-source
  two-stage sort:
  - source candidates with `anchor`, `current_diff`, `lexical`, or `symbol`
    signals are sorted first;
  - remaining candidates are sorted after that by local metadata score.
- Keep tests out of the protected source floor so validation/test selection
  remains handled by the existing selection budget logic.
- Add a planner-level test proving `.ctxhelm/provider-policy.json` with
  `enableLocalMetadataReranker: true` allows and applies the local metadata
  reranker through `prepare_context_plan`.
- Preserve the default-off provider policy. This does not enable reranking by
  default.

## Proof

Artifacts:

- `.ctxhelm/e2e/phase259-runtime-reranker-floor-refactoringminer.json`
- `.ctxhelm/e2e/phase259-runtime-reranker-floor-ctxhelm.json`

Commands:

```bash
cargo test -p ctxhelm-compiler local_metadata_reranker --locked
cargo test -p ctxhelm-compiler prepare_plan_applies_policy_enabled_local_metadata_reranker --locked
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase259-runtime-reranker-floor-refactoringminer.json
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase259-runtime-reranker-floor-ctxhelm.json
```

RefactoringMiner:

```text
decision: hold
namedRegressions: []
rerankerContribution.targetHitDelta: 0
rerankerContribution.improvedCommitCount: 0
rerankerContribution.regressedCommitCount: 0
rerankerContribution.neutralCommitCount: 10
diagnostic: reranker_contribution_neutral
```

ctxhelm:

```text
decision: hold
namedRegressions: []
rerankerContribution.targetHitDelta: +16
rerankerContribution.improvedCommitCount: 9
rerankerContribution.regressedCommitCount: 0
rerankerContribution.defaultOnlyTargetHitCount: 2
rerankerContribution.protectedEvidenceTargetMissRateDelta: -0.51428574
diagnostics: reranker_contribution_target_lift, reranker_contribution_protected_target_improved, reranker_contribution_target_churn
```

## Decision

Promote the runtime safety guard, not default reranking.

Policy-enabled runtime reranking now honors a protected source floor, while
the report still shows corpus-specific behavior and target churn. Reranking
remains opt-in. The next R&D step can safely focus on query-family routing or
learned fusion because the opt-in runtime path no longer has a known protected
source displacement gap.
