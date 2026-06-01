# Phase 100: Resource-Backed Gap Summaries

## Goal

Make remaining retrieval-gap summaries actionable for agents without changing
top-10 ranking or exposing source text.

After Phase 99, broad context-area resources had enough structure for native
progressive reads, but historical eval gap summaries still only reported
`examplePaths`. Agents and maintainers could see `area_context_only` or
ranked-below-budget gaps, but not the matching context-area resource to open
next.

## Implementation

- Added additive source-free fields to `RetrievalGapSummary`:
  - `contextArea`
  - `contextAreaResourceUri`
  - `nextReadPaths`
- Populated these fields for grouped gap summaries using path-derived context
  areas and bounded path lists.
- Kept source text out of proof reports.
- Kept ranking, candidate generation, MCP tools, and validation channels
  unchanged.

## Evidence

Focused tests:

```bash
cargo test -p ctxhelm-compiler retrieval_gap_summaries_skip_validation_covered_tests -- --nocapture
cargo test -p ctxhelm-compiler ablation_historical_eval_groups_source_free_retrieval_gaps -- --nocapture
```

Broad proof:

```bash
cargo run --release -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > .ctxhelm/e2e/phase100-resource-backed-gap-summaries-proof.json

python3 scripts/check-product-proof.py \
  .ctxhelm/e2e/phase100-resource-backed-gap-summaries-proof.json
```

Committed proof:

- `.ctxhelm/e2e/phase100-resource-backed-gap-summaries-proof.json`

Result:

- `releaseGate.decision = promote`
- No metric deltas versus Phase 99 for File Recall@10, Source Recall@10, Test
  Recall@10, Effective Validation Recall@10, or broad context-area recall across
  the four-repo proof.
- VeriSchema `area_context_only` and ranked-below-budget gap summaries now carry
  resource URIs such as
  `ctxhelm://repo/context-area/schema_agent%2Fcore`.
- ctxhelm remains File Recall@10 `0.47460318`, Source Recall@10 `0.7166667`,
  and broad context-area recall `1.0`.
- VeriSchema remains File Recall@10 `0.18449473`, Source Recall@10
  `0.31067252`, Test Recall@10 `0.7089947`, Effective Validation Recall@10
  `1.0`, and broad context-area recall `0.71851856`.

## Notes

This phase turns the remaining gap report from a diagnostic into a progressive
read plan. It intentionally avoids target-file churn because previous top-10
experiments regressed production proof metrics.
