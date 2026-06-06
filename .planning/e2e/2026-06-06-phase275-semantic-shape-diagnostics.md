# Phase 275 - Semantic Shape Diagnostics

## Goal

Move beyond separate query-family and path-family diagnostics for
`semantic_corroborated_reranked` by measuring source-free query/path shapes.
The hypothesis was that the RefactoringMiner lift might be separable as a
Java/MCP/package-shaped route without inheriting ctxhelm, ReAgent, or
VeriSchema regressions.

## Code Kept

Added `semanticCorroboratedRerankerContribution.shapeContributions` to the
semantic/precision gate report.

Each shape groups target movement by:

- `queryFamily`
- `pathFamily`
- `rerankerOnlyTargetHitCount`
- `defaultOnlyTargetHitCount`
- `targetHitDelta`
- `routingRecommendation`
- example reranker-only/default-only paths

This is source-free report metadata only. It does not change default ranking,
planner behavior, semantic documents, MCP behavior, or provider policy.

## Temporary Experiment Rejected

A temporary eval-only `semantic_shape_routed_reranked` variant was tested and
then removed. The variant started from default ranking and tried to insert only
semantic-corroborated candidates matching narrow shapes:

- `symbol_identifier/docs`
- `commit_clue/config`
- `domain_phrase/java_source`
- `domain_phrase/java_test`

The stricter route did not preserve the useful RefactoringMiner lift:

| Repo | Default | Full semantic-corroborated | Temporary shape-routed | Shape-routed result |
| --- | ---: | ---: | ---: | --- |
| RefactoringMiner | `0.41857147` | `0.5619047` | `0.40023813` | regress `-2` target hits |
| ctxhelm | `0.44620585` | `0.44333735` | `0.44620585` | neutral |
| ReAgent | `0.35` | `0.325` | `0.325` | regress `-1` target hit |
| VeriSchema | `0.39382353` | `0.36960787` | `0.39382353` | neutral |

The failure mode is budget displacement: even narrow allowed semantic candidates
can displace default target tests or source files. On RefactoringMiner, the
temporary route lost `src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpToolsTest.java`
in two commits. On ReAgent, it lost
`tests/test_run_second_family_end_to_end.py`.

Decision: reject shape-routed semantic-corroborated insertion as a runtime or
standing eval variant.

## Final Four-Repo Diagnostics

Final diagnostics-only gates:

```bash
cargo test -p ctxhelm-compiler --locked
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase275-semantic-shape-diagnostics-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase275-semantic-shape-diagnostics-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase275-semantic-shape-diagnostics-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase275-semantic-shape-diagnostics-verischema.json
```

Final `semantic_corroborated_reranked` results:

| Repo | Default | Semantic-corroborated | Target delta | Regressed commits | Default-only targets |
| --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.41857147` | `0.5619047` | `+5` | `0` | `0` |
| ctxhelm | `0.44620585` | `0.44333735` | `-7` | `9` | `37` |
| ReAgent | `0.35` | `0.325` | `-4` | `3` | `6` |
| VeriSchema | `0.39382353` | `0.36960787` | `-8` | `3` | `10` |

Clean RefactoringMiner route shapes:

| Query family | Path family | Delta |
| --- | --- | ---: |
| `symbol_identifier` | `docs` | `+2` |
| `commit_clue` | `config` | `+1` |
| `domain_phrase` | `java_source` | `+1` |
| `domain_phrase` | `java_test` | `+1` |

Cross-repo blockers:

- ctxhelm: `domain_phrase/rust_source -7`, `broad_scope/rust_source -11`,
  and `broad_scope/planning -12` churn.
- ReAgent: `symbol_identifier/scripts -1`,
  `symbol_identifier/planning -4`.
- VeriSchema: `domain_phrase/python_source -1`,
  `broad_scope/python_source -9`.

## Decision

Keep shape diagnostics. Reject shape-routed semantic-corroborated insertion.

The RefactoringMiner lift is real but still not safely routable with this
simple query/path-shape separator. The next semantic work should not add another
handwritten route. It should inspect the actual regressing commits and compare
candidate insertion against target-test/source preservation constraints, or
move to learned/listwise budget allocation with explicit default-target
protection.
