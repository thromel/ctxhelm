# Phase 281 - Learned Semantic Policy Holdout

## Goal

Evaluate the Phase 280 learned semantic policy shape on held-out commits before
any runtime/default promotion. The held-out variant must remain source-free and
must not reuse the current commit as evidence for its own profile.

## Code Kept

Added eval-only `semantic_learned_policy_holdout_reranked`.

The variant reuses the learned-profile semantic reranker machinery, but applies
the exported policy support threshold during leave-one-out evaluation:

- key: source-free `(query_family, path_family)`
- holdout strategy: leave one commit out, train profile support on the rest
- minimum support: `minimumSupportCommitCount = 2`
- candidate source: semantic-corroborated ranking only
- runtime behavior: unchanged
- promotion status: `runtimePromotable = false`

Added gate fields:

- `learnedSemanticPolicyHoldout`
- `learnedPolicySemanticHoldoutContribution`

Added gate diagnostics:

- `semantic_learned_policy_holdout_regression`
- `semantic_learned_policy_holdout_clean_lift`
- `semantic_learned_policy_holdout_neutral`
- `semantic_learned_policy_holdout_insufficient`

## Four-Repo Gate

Commands:

```bash
cargo test -p ctxhelm-compiler learned_policy --locked
cargo test -p ctxhelm-compiler semantic --locked
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase281-learned-policy-holdout-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase281-learned-policy-holdout-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase281-learned-policy-holdout-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase281-learned-policy-holdout-verischema.json
```

Results:

| Repo | Default | Learned-profile | Holdout | Holdout applied commits | Target delta | Regressed commits | Holdout decision |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | `0.41857147` | `0.51857144` | `0.41857147` | `0` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| ctxhelm | `0.44620585` | `0.44620585` | `0.44620585` | `0` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| ReAgent | `0.35` | `0.35` | `0.35` | `0` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| VeriSchema | `0.39382353` | `0.39382353` | `0.39382353` | `0` | `0` | `0` | `insufficient_eligible_holdout_profiles` |

## Decision

Keep the holdout artifact and diagnostics. The result is useful negative
evidence: the only Phase 280 eligible durable profile is too sparse for the
current leave-one-out support threshold. RefactoringMiner's full learned-profile
lift still exists, but it does not survive as an exported policy when each
commit is held out and `minimumSupportCommitCount = 2` is enforced.

Runtime/default promotion remains blocked. The next semantic R&D step should
use a broader training/holdout corpus or separate revision slices so exported
profiles can accumulate repeated support outside the held-out commits.
