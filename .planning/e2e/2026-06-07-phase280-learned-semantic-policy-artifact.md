# Phase 280 - Learned Semantic Policy Artifact

## Goal

Turn Phase 278's eval-only learned-profile semantic reranker into a durable
source-free policy artifact shape before any runtime promotion. The artifact
must expose support thresholds, staleness status, and holdout status so a
future runtime policy cannot silently treat a same-gate experiment as a stable
default.

## Code Kept

Added `learnedSemanticPolicy` to `SemanticPrecisionGateReport`.

The artifact records:

- schema version and deterministic policy id
- source variant: `semantic_corroborated_reranked`
- source eval range and ranking budget
- `minimumSupportCommitCount = 2`
- profile counts and per-profile eligibility/block reasons
- source-free `(query_family, path_family)` profile rows
- staleness status scoped to the measured eval snapshot
- holdout status requiring cross-repo proof
- `defaultEligible = false`
- `runtimePromotable = false`

Added gate diagnostics:

- `semantic_learned_policy_artifact_created`
- `semantic_learned_policy_no_eligible_profiles`
- `semantic_learned_policy_holdout_required`

This does not change runtime planning, MCP behavior, provider policy, pack
selection, or the Phase 278 eval-only ranking variant.

## Four-Repo Gate

Commands:

```bash
cargo test -p ctxhelm-compiler learned_semantic_policy --locked
cargo test -p ctxhelm-compiler semantic --locked
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase280-learned-policy-artifact-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase280-learned-policy-artifact-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase280-learned-policy-artifact-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase280-learned-policy-artifact-verischema.json
```

Results:

| Repo | Default | Learned-profile | Target delta | Regressed commits | Policy profiles | Eligible profiles | Runtime promotable |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | `0.41857147` | `0.51857144` | `+2` | `0` | `10` | `1` | `false` |
| ctxhelm | `0.44620585` | `0.44620585` | `0` | `0` | `10` | `0` | `false` |
| ReAgent | `0.35` | `0.35` | `0` | `0` | `11` | `0` | `false` |
| VeriSchema | `0.39382353` | `0.39382353` | `0` | `0` | `10` | `0` | `false` |

The only eligible durable profile is RefactoringMiner
`symbol_identifier/docs`, observed in `2` commits with `2` inserted target
hits, `0` inserted non-targets, and `0` lost default targets.

## Decision

Keep the artifact and diagnostics. This closes the immediate "durable
source-free policy artifact" gap from Phase 278, but it does not complete
semantic runtime/default promotion.

The next semantic R&D step should train on one corpus or revision slice and
evaluate the exported `learnedSemanticPolicy` artifact on a separate holdout
slice/repo. Runtime promotion remains blocked until that cross-repo holdout
proof shows no regressions under the same support and staleness contract.
