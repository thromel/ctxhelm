# Phase 186: Gap Profile Deduplication

## Goal

Keep retrieval-gap `missedCount` faithful to the number of missed files while
preventing source-free context-area profile fields from being merged repeatedly
when several missed files collapse into one grouped gap summary.

## Implementation

- Added a per-summary merge key for context-area profile aggregation.
- Kept `missedCount`, `examplePaths`, and `nextReadPaths` file-count based.
- Deduplicated only these additive profile fields:
  - `contextAreaSignalCounts`
  - `contextAreaRoleCounts`
  - `contextAreaSelectedRoleCounts`
  - `contextAreaUnselectedCount`
- Added a focused regression test where two missed source files share one
  grouped area gap; the summary still reports `missedCount = 2`, but the area
  profile counts are merged once.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-compiler \
  retrieval_gap_summaries_skip_validation_covered_tests \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  ablation_historical_eval_groups_source_free_retrieval_gaps \
  --locked -- --nocapture
```

Result: both focused tests passed.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase186-gap-profile-dedupe-release-home \
  /tmp/ctxhelm-rd/phase186-gap-profile-dedupe-release-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase186-gap-profile-dedupe-release-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase186-gap-profile-dedupe-release-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase186-gap-profile-dedupe-release-proof.json
```

Result:

- `releaseGate.decision = promote`
- RefactoringMiner: `match`, runtime `3219ms`
- ctxhelm: `beat`, runtime `4433ms`
- ReAgent: `beat`, runtime `5662ms`
- VeriSchema: `beat`, runtime `5458ms`

## Runtime Note

A debug `cargo run` cold proof blocked on RefactoringMiner runtime
(`24898ms > 15000ms`) while quality verdicts stayed unchanged. The failing
bucket was planner execution time in the debug binary, and RefactoringMiner had
zero retrieval-gap summaries in this phase, so the new diagnostic dedupe path
was not the runtime cause. The release gate proves the selected archive/release
binary; the optimized release proof above is therefore the relevant product
runtime evidence for this phase.
