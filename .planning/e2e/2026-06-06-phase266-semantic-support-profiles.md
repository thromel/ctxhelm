# Phase 266 - Semantic Support Profiles

## Scope

Follow Phase 265's semantic family stability rejection with a stricter
within-family diagnostic: for every semantic-only target and semantic-only
non-target, record whether another source-free retrieval signal also selected
that path.

The question was whether "semantic plus corroboration" can separate useful
semantic hits from same-family semantic noise.

## Implementation

- Added `SemanticOnlySupportProfile`.
- Extended `semanticContribution.queryFamilyContributions` with:
  - `semanticOnlyTargetWithNonsemanticSupportCount`
  - `semanticOnlyTargetWithoutNonsemanticSupportCount`
  - `semanticOnlyNonTargetWithNonsemanticSupportCount`
  - `semanticOnlyNonTargetWithoutNonsemanticSupportCount`
  - `supportProfiles`
- Added support-aware diagnostics:
  - `semantic_query_family_unsupported_target_hold`
  - `semantic_query_family_supported_noise_hold`
  - `semantic_query_family_supported_mixed_hold`
- Updated Markdown gate rendering and benchmarking docs.

## Commands

```bash
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase266-semantic-support-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase266-semantic-support-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase266-semantic-support-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase266-semantic-support-verischema.json
```

## Evidence

| Repo | Decision | Default Recall@10 | Semantic Recall@10 | Finding |
| --- | --- | ---: | ---: | --- |
| ctxhelm | `hold` | `0.44620585` | `0.44620585` | `broad_scope` and `domain_phrase` both emit `semantic_query_family_supported_mixed_hold`: semantic-only targets and semantic-only non-targets are both corroborated. |
| ReAgent | `hold` | `0.35` | `0.35` | `commit_clue`, `symbol_identifier`, and `domain_phrase` all emit `semantic_query_family_supported_noise_hold`: corroborated semantic-only non-targets with no corroborated semantic-only targets. |
| RefactoringMiner | `hold` | `0.41857147` | `0.4285714` | `symbol_identifier` emits `semantic_query_family_supported_mixed_hold`; `commit_clue` emits `semantic_query_family_supported_noise_hold`. |
| VeriSchema | `block` | `0.39382353` | `0.39382353` | `domain_phrase` and `broad_scope` emit `semantic_query_family_supported_mixed_hold`; `symbol_identifier` emits `semantic_query_family_supported_noise_hold`. |

## Result

Generic semantic corroboration is not a safe routing rule. In the Phase 266
slice, semantic-only non-targets are also corroborated by source-free signals,
and several families have both corroborated target hits and corroborated noise.

This rules out a simple "semantic + any other signal" runtime policy. The next
semantic R&D should either:

- test support-family-specific constraints, such as excluding support profiles
  that repeatedly carry only non-targets, or
- change local semantic query/document construction so semantic-only targets
  repeat without relying on noisy generic corroboration.

Semantic remains opt-in.
