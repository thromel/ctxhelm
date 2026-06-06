# Phase 278 - Learned-Profile Semantic Reranker

## Goal

Test the next semantic R&D step after Phase 277 rejected another hand-written
budget rule. The hypothesis was that a source-free learned profile policy could
admit semantic-corroborated candidates only when other commits in the same gate
show that the same query/path profile adds target hits without inserted
non-targets or lost default targets.

## Code Kept

Added eval-only `semantic_learned_profile_reranked`.

The variant is leave-one-out by commit:

- build profile evidence from all other measured commits
- profile key is source-free `(query_family, path_family)`
- route only profiles with inserted semantic target hits
- block profiles with any inserted non-targets
- block profiles with any lost default target hits
- leave the current commit out of its own profile evidence

This is a report/eval variant only:

- no default ranking change
- no runtime provider-policy change
- no MCP/planner/pack behavior change
- no cloud behavior
- no source text logging

Added report field:

- `learnedProfileSemanticRerankerContribution`

Added diagnostics:

- `semantic_learned_profile_reranker_clean_lift`
- `semantic_learned_profile_reranker_regression`
- `semantic_learned_profile_reranker_neutral`

## Four-Repo Gate

Commands:

```bash
cargo test -p ctxhelm-compiler semantic --locked
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase278-learned-profile-semantic-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase278-learned-profile-semantic-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase278-learned-profile-semantic-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase278-learned-profile-semantic-verischema.json
```

Results:

| Repo | Default | Semantic-corroborated | Family-budget | Learned-profile | Target delta | Regressed commits | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | `0.41857147` | `0.5619047` | `0.43690476` | `0.51857144` | `+2` | `0` | promising eval-only lift |
| ctxhelm | `0.44620585` | `0.44333735` | `0.37543336` | `0.44620585` | `0` | `0` | neutral, no regression |
| ReAgent | `0.35` | `0.325` | `0.4` | `0.35` | `0` | `0` | neutral, no regression |
| VeriSchema | `0.39382353` | `0.36960787` | `0.34382352` | `0.39382353` | `0` | `0` | neutral, no regression |

## Decision

Keep the eval-only learned-profile variant and diagnostics. This is the first
post-Phase-270 semantic reranker experiment that preserves the default score on
ctxhelm, ReAgent, and VeriSchema while recovering clean RefactoringMiner lift.

Do not promote to runtime yet. The experiment proves the direction is better
than another handwritten route, but runtime exposure would require a durable
source-free profile-training/export contract, staleness handling, minimum
support thresholds, and a broader no-regress proof.

The next semantic R&D step should turn this into an explicit learned-policy
artifact with cross-repo holdout gates, or test whether a stricter minimum
support threshold keeps the RefactoringMiner lift while remaining neutral on the
other corpora.
