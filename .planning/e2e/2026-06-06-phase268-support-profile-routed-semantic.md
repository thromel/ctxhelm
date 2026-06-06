# Phase 268 - Support-Profile-Routed Semantic Variant

## Scope

Test the Phase 267 hypothesis that semantic-only candidates can be routed by
specific `(query family, support family)` profiles. This is an eval-only
experiment. It does not change runtime provider policy, planner behavior, or
default semantic promotion.

## Implementation

Added `support_profile_routed_semantic` to `ctxhelm eval gate`.

The variant:

- starts from the default ranking;
- learns route-candidate support profiles from `semanticContribution`;
- inserts semantic-only files only when their query family and non-semantic
  support family had semantic-only target hits and no semantic-only non-targets;
- excludes support-profile mixed/noise holds;
- recomputes ranking metrics, token ROI, protected evidence selection, and
  named regressions for the synthetic report.

## Validation Commands

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler support_profile_routed_semantic --locked
cargo test -p ctxhelm-compiler semantic_contribution --locked
cargo build -p ctxhelm --features local-embeddings
```

Four-repo gate refresh:

```bash
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase268-support-profile-routed-semantic-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase268-support-profile-routed-semantic-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase268-support-profile-routed-semantic-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase268-support-profile-routed-semantic-verischema.json
```

## Results

| Repo | Decision | Default Recall@10 | Local Semantic Recall@10 | Support-Profile-Routed Recall@10 | Routed regressions | Routed wins |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| ctxhelm | `hold` | `0.44620585` | `0.44620585` | `0.44620585` | `0` | `0` |
| ReAgent | `hold` | `0.35` | `0.35` | `0.35` | `0` | `0` |
| RefactoringMiner | `hold` | `0.41857147` | `0.4285714` | `0.41857147` | `0` | `0` |
| VeriSchema | `block` | `0.39382353` | `0.39382353` | `0.39382353` | `0` | `0` |

Support-profile routing is safe in this run, but it does not improve measured
top-10 retrieval. It also does not preserve RefactoringMiner's small
`local_semantic` lift.

## Decision

Reject support-profile routing as a promotion path for now.

The next semantic R&D should not keep routing the existing clean-looking
profiles. It should target better local query construction, local model choice,
or fusion features that create repeated target lift before routing or runtime
policy exposure.
