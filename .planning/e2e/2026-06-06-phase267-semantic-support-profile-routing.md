# Phase 267 - Semantic Support-Profile Routing Diagnostics

## Scope

Phase 266 rejected generic "semantic plus any other signal" routing. Phase 267
adds the next narrower diagnostic layer: classify each semantic-only support
profile inside a query family as a route candidate, mixed hold, or noise hold.

This is still eval/reporting only. No runtime semantic routing was enabled.

## Implementation

- Added support-profile diagnostics:
  - `semantic_support_profile_route_candidate`
  - `semantic_support_profile_mixed_hold`
  - `semantic_support_profile_noise_hold`
- Added focused compiler tests covering target-only, mixed, and noise support
  profile cases.
- Updated `docs/benchmarking.md` with the new diagnostic contract.

## Commands

```bash
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase267-semantic-support-profile-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase267-semantic-support-profile-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase267-semantic-support-profile-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase267-semantic-support-profile-verischema.json
```

## Evidence

| Repo | Decision | Default Recall@10 | Semantic Recall@10 | Profile route candidates | Mixed holds | Noise holds |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| ctxhelm | `hold` | `0.44620585` | `0.44620585` | `1` | `1` | `3` |
| ReAgent | `hold` | `0.35` | `0.35` | `0` | `0` | `8` |
| RefactoringMiner | `hold` | `0.41857147` | `0.4285714` | `0` | `2` | `6` |
| VeriSchema | `block` | `0.39382353` | `0.39382353` | `3` | `2` | `13` |

Observed route candidates:

- ctxhelm `broad_scope` + `lexical_expansion_co_change`: `2` semantic-only
  target hits and `0` semantic-only non-targets.
- VeriSchema `domain_phrase` + `lexical_dependency`: `1` semantic-only target
  hit and `0` semantic-only non-targets.
- VeriSchema `domain_phrase` + `lexical_expansion`: `1` semantic-only target
  hit and `0` semantic-only non-targets.
- VeriSchema `broad_scope` + `lexical_co_change`: `1` semantic-only target hit
  and `0` semantic-only non-targets.

## Result

Support-profile diagnostics are useful, but the evidence is not strong enough
for runtime semantic routing. Route candidates exist, but they are sparse and
surrounded by profile-specific noise. ReAgent remains a strong counterexample:
the same diagnostic layer finds only noise holds there.

The next semantic R&D should test a measured eval variant that can include only
support-profile route candidates and explicitly exclude support-profile noise
holds, then compare recall, churn, and named regressions against default before
any provider-policy exposure.
