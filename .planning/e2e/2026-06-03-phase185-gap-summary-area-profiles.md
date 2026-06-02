# Phase 185: Gap Summary Area Profiles

## Goal

Make retrieval-gap summaries carry the same source-free area signal profile that
Phase 184 added to task-conditioned `contextAreas`.

Phase 184 made broad context-area guidance explainable to agents, but the
historical proof still showed gap summaries such as `ranked_below_budget_*` and
`area_context_only` without the matching area signal mix. That forced future R&D
to cross-reference commit-level `contextAreas` by hand.

## Implementation

- Added additive source-free fields to `RetrievalGapSummary`:
  - `contextAreaSignalCounts`
  - `contextAreaRoleCounts`
  - `contextAreaSelectedRoleCounts`
  - `contextAreaUnselectedCount`
- Populated those fields from the matching task-conditioned `ContextArea` on
  each historical commit row.
- Aggregated the counts when a gap summary groups multiple missed paths.
- Kept all fields defaulted and omitted when no matching context area exists,
  preserving compatibility with older reports and non-area gaps.
- Preserved the existing validation-test behavior: tests covered by
  `recommendedTests` or validation commands are still not counted as unresolved
  retrieval gaps.

The fields are diagnostic only. They do not change ranking, retrieval budgets,
or release-gate thresholds.

## Focused Verification

```bash
cargo test -p ctxhelm-compiler retrieval_gap_summaries_skip_validation_covered_tests --locked -- --nocapture
cargo test -p ctxhelm-compiler ablation_historical_eval_groups_source_free_retrieval_gaps --locked -- --nocapture
```

Result: both focused tests passed.

## Product Proof

Command:

```bash
rm -rf /tmp/ctxhelm-phase185-gap-profile-home \
  /tmp/ctxhelm-rd/phase185-gap-summary-profile-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase185-gap-profile-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase185-gap-summary-profile-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase185-gap-summary-profile-proof.json
```

Result:

- `releaseGate.decision = promote`
- RefactoringMiner: `0` gap summaries
- ctxhelm: `7` gap summaries, `7` with area profiles
- ReAgent: `0` gap summaries
- VeriSchema: `10` gap summaries, `9` with area profiles

Sample profiled gaps:

- ctxhelm `crates/ctxpack-compiler`
  - `signalGap = ranked_below_budget_co_change`
  - `contextAreaSignalCounts = co_change=19, dependency=3, lexical=2, lexical_expansion=9`
  - `contextAreaRoleCounts = source=26`
  - `contextAreaSelectedRoleCounts = source=7`
  - `contextAreaUnselectedCount = 19`
- ctxhelm `docs`
  - `signalGap = lexical_only_miss`
  - `contextAreaSignalCounts = co_change=14, docs=22, lexical=4, lexical_expansion=10`
  - `contextAreaRoleCounts = docs=22`
  - `contextAreaSelectedRoleCounts = docs=2`
  - `contextAreaUnselectedCount = 20`
- VeriSchema `schema_agent/agents`
  - `signalGap = ranked_below_budget_dependency`
  - `contextAreaSignalCounts = co_change=3, dependency=18, lexical_expansion=15`
  - `contextAreaRoleCounts = source=30`
  - `contextAreaSelectedRoleCounts = source=3`
  - `contextAreaUnselectedCount = 27`
- VeriSchema `schema_agent/nlp`
  - `signalGap = area_context_only`
  - `contextAreaSignalCounts = lexical_expansion=8`
  - `contextAreaRoleCounts = source=8`
  - `contextAreaSelectedRoleCounts` omitted because no source was selected
  - `contextAreaUnselectedCount = 8`

One VeriSchema docs gap under `paper/pvldb` remained unprofiled because that
docs area was not present in the task-conditioned `contextAreas`; the summary
still includes its source-free context-area URI and next-read path.

## Decision

Accept the instrumentation. It turns the latest proof into a better R&D
debugging artifact without perturbing top-10 recall or validation evidence.
