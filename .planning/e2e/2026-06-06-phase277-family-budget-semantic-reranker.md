# Phase 277 - Family-Budget Semantic Reranker

## Goal

Test the next target-preservation idea after Phase 276 showed semantic
displacement pressure. The hypothesis was that semantic-corroborated candidates
might be safer if they could only reuse the default top-K path-family budget,
instead of expanding docs/planning/scripts/paper families at the expense of
source or validation families.

## Code Kept

Added eval-only `semantic_family_budget_reranked`.

The variant starts from the semantic-corroborated ranking, but admits a path
only when its source-free path family has remaining budget from the default
top-K ranking. It then fills remaining allowed family slots from the default
ranking. This is a report/eval variant only:

- no default ranking change
- no runtime provider-policy change
- no MCP/planner/pack behavior change
- no cloud behavior
- no source text logging

Added report field:

- `familyBudgetSemanticRerankerContribution`

Added diagnostics:

- `semantic_family_budget_reranker_clean_lift`
- `semantic_family_budget_reranker_regression`
- `semantic_family_budget_reranker_neutral`

## Four-Repo Gate

Commands:

```bash
cargo test -p ctxhelm-compiler --locked
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase277-family-budget-semantic-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase277-family-budget-semantic-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase277-family-budget-semantic-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase277-family-budget-semantic-verischema.json
```

Results:

| Repo | Default | Semantic-corroborated | Family-budget | Target delta | Regressed commits | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | `0.41857147` | `0.5619047` | `0.43690476` | `+2` | `0` | useful but weaker than full semantic-corroborated |
| ctxhelm | `0.44620585` | `0.44333735` | `0.37543336` | `-10` | `9` | rejected |
| ReAgent | `0.35` | `0.325` | `0.4` | `+1` | `0` | useful on this corpus |
| VeriSchema | `0.39382353` | `0.36960787` | `0.34382352` | `-1` | `1` | rejected |

## Decision

Keep the eval-only family-budget variant and diagnostics as R&D evidence, but
reject promotion. The constraint is better than the Phase 276 naive preservation
prototype because it recovers clean lift on RefactoringMiner and ReAgent, but it
still regresses ctxhelm and VeriSchema.

The next semantic R&D should not add another hand-written budget rule. The
remaining path is learned/listwise allocation with explicit no-regress
constraints, or a narrower corpus/profile-specific policy that can explain why
ctxhelm broad planning/doc tasks and VeriSchema docs/Python-source tasks should
be held.
