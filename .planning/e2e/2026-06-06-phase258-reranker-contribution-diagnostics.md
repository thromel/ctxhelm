# Phase 258: Reranker Contribution Diagnostics

## Goal

Make safe local metadata reranker behavior inspectable enough to support the
next routing/default decision.

Phase 257 made the eval-only reranker preserve protected source evidence and
validation-test reserves. The remaining question is not whether the reranker is
safe enough to evaluate; it is where it helps, where it is neutral, and whether
it swaps out any default target hits while producing net lift.

## Change

- Add `rerankerContribution` to semantic/precision gate reports.
- Count improved, regressed, and neutral commits for `local_metadata_reranked`
  versus `ctxhelm_default`.
- Count default target hits, reranked target hits, target-hit delta,
  reranker-only target hits, and default-only target hits.
- Report protected evidence miss-rate deltas for the reranked variant.
- Emit source-free reranker diagnostics:
  - `reranker_contribution_target_lift`
  - `reranker_contribution_neutral`
  - `reranker_contribution_regressed_commits`
  - `reranker_contribution_protected_target_improved`
  - `reranker_contribution_target_churn`
- Render the same contribution summary in Markdown gate output.

## Proof

Artifacts:

- `.ctxhelm/e2e/phase258-reranker-contribution-refactoringminer.json`
- `.ctxhelm/e2e/phase258-reranker-contribution-ctxhelm.json`

Commands:

```bash
cargo test -p ctxhelm-compiler reranker_contribution --locked
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase258-reranker-contribution-refactoringminer.json
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase258-reranker-contribution-ctxhelm.json
```

RefactoringMiner reranker contribution:

```text
evaluatedCommits: 10
improvedCommitCount: 0
regressedCommitCount: 0
neutralCommitCount: 10
targetHitDelta: 0
rerankerOnlyTargetHitCount: 0
defaultOnlyTargetHitCount: 0
protectedEvidenceMissRateDelta: -0.14285715
protectedEvidenceTargetMissRateDelta: 0.0
diagnostic: reranker_contribution_neutral
```

ctxhelm reranker contribution:

```text
evaluatedCommits: 10
improvedCommitCount: 8
regressedCommitCount: 0
neutralCommitCount: 2
targetHitDelta: +14
rerankerOnlyTargetHitCount: 16
defaultOnlyTargetHitCount: 2
protectedEvidenceMissRateDelta: -0.37113398
protectedEvidenceTargetMissRateDelta: -0.516129
diagnostics: reranker_contribution_target_lift, reranker_contribution_protected_target_improved, reranker_contribution_target_churn
```

The default-only churn is source-free and explicit:

```text
afc6bfe9b149: .planning/STATE.md
1de0b55c6874: .planning/STATE.md
```

## Decision

Promote the diagnostics, not unconditional reranking.

The new report proves the safe metadata reranker has a strong ctxhelm slice but
is neutral on RefactoringMiner. It also exposes target churn inside the winning
ctxhelm slice, which means the next R&D step should be query-family routing or
learned fusion with churn constraints, not a blanket default.
