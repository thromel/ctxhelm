# Phase 276 - Reranker Displacement Diagnostics

## Goal

Explain the Phase 275 failure mode without changing runtime ranking. Previous
semantic-corroborated diagnostics showed target gains and losses, but did not
show which reranked-only top-K files entered the budget when default target
files were lost.

## Code Kept

Added `RerankerContributionSummary.displacementContributions`.

Each row is source-free and groups default-target loss pressure by:

- `queryFamily`
- `insertedPathFamily`
- selected-only target and non-target file counts
- lost target file count
- lost target path-family counts
- example inserted paths
- example lost target paths

The field is defaulted for backward JSON compatibility. It is report metadata
only. It does not change default ranking, semantic documents, planner behavior,
MCP behavior, provider policy, or context packing.

Added diagnostic:

- `semantic_corroborated_displacement_pressure`

This diagnostic fires when semantic-corroborated reranking inserted non-target
top-K files while default target files were lost.

## Four-Repo Gate

Commands:

```bash
cargo test -p ctxhelm-compiler --locked
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase276-reranker-displacement-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase276-reranker-displacement-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase276-reranker-displacement-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase276-reranker-displacement-verischema.json
```

Results for `semantic_corroborated_reranked`:

| Repo | Default | Semantic-corroborated | Target delta | Regressed commits | Default-only targets | Displacement pressure |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | `0.41857147` | `0.5619047` | `+5` | `0` | `0` | none |
| ctxhelm | `0.44620585` | `0.44333735` | `-7` | `9` | `37` | planning/scripts/docs displaced planning, docs, and Rust source targets |
| ReAgent | `0.35` | `0.325` | `-4` | `3` | `6` | docs/paper/planning displaced planning and script targets |
| VeriSchema | `0.39382353` | `0.36960787` | `-8` | `3` | `10` | docs/scripts/paper displaced Python source targets |

Top displacement rows:

- ctxhelm: `broad_scope/planning` inserted `20` non-targets and lost
  planning/docs/Rust-source targets; `broad_scope/scripts` inserted `14`
  non-targets; `domain_phrase/planning` displaced Rust source targets.
- ReAgent: `symbol_identifier/docs` inserted `5` non-targets and displaced
  planning targets; `symbol_identifier/other` inserted paper files and
  displaced planning/script targets.
- VeriSchema: `broad_scope/docs` inserted `8` non-target docs and displaced
  Python source targets; `broad_scope/scripts` inserted scripts while Python
  source targets were lost.

## Decision

Keep displacement diagnostics. Do not promote another handwritten semantic
route.

This confirms the next semantic R&D step should be target-preserving budget
constraints or learned/listwise allocation with explicit no-regress protection.
The problem is not only identifying clean-looking semantic shapes; it is
preserving default target tests/source files when semantic candidates enter the
fixed top-K budget.
