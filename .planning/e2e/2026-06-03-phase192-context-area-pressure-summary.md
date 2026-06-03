# Phase 192: Context Area Pressure Summary

## Goal

Phase 191 made each task-conditioned context area explain its inspection
pressure. This phase makes that signal useful at eval/proof scale by aggregating
pressure across historical commits and repositories. The goal is source-free
diagnostics for broad-area usefulness, not ranking churn.

## Implementation

- Added `contextAreaPressureSummary` to `HistoricalEvalReport`.
- Aggregated:
  - total context-area count
  - zero-selected area count
  - total inspection pressure
  - source-like, validation, and docs pressure
  - highest-pressure area with URI, coverage, pressure, and unselected count
- Rendered the summary in historical eval markdown and benchmark-suite markdown.
- Extended `scripts/check-product-proof.py` to validate emitted pressure-summary
  totals and source-free status.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-compiler broad_context_area_recall_counts_surfaced_areas --locked -- --nocapture
cargo test -p ctxhelm-compiler historical_eval_report_public_json_shape_is_stable --locked -- --nocapture
cargo test -p ctxhelm historical_eval_report_renders_source_free_metrics --locked -- --nocapture
cargo test -p ctxhelm product_proof_checker_accepts_promote_and_rejects_block --locked -- --nocapture
```

Result: passed.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase192-area-pressure-summary-home \
  /tmp/ctxhelm-rd/phase192-area-pressure-summary-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase192-area-pressure-summary-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase192-area-pressure-summary-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase192-area-pressure-summary-proof.json
```

Result: `releaseGate.decision = promote`.

Pressure summary from the proof:

| Repo | Areas | Total Pressure | Source-Like | Validation | Docs | Highest Pressure Area |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | 4 | 48 | 48 | 0 | 0 | `src/main/java/org/refactoringminer/mcp` |
| ctxhelm | 74 | 456 | 366 | 0 | 90 | `crates/ctxpack-index` |
| ReAgent | 12 | 166 | 165 | 0 | 1 | `ccia/llm` |
| VeriSchema | 54 | 666 | 651 | 6 | 9 | `schema_agent/evaluation` |

Metric comparison against Phase 191:

| Repo | File Recall@10 | Source Recall@10 | Test Recall@10 | Broad Area Recall |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8 -> 0.8` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| ctxhelm | `0.67777777 -> 0.67777777` | `0.55 -> 0.55` | `0.0 -> 0.0` | `1.0 -> 1.0` |
| ReAgent | `0.8 -> 0.8` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| VeriSchema | `0.35529414 -> 0.35529414` | `0.5277778 -> 0.5277778` | `0.7896825 -> 0.7896825` | `0.5777778 -> 0.5777778` |

## Why It Matters

The aggregate shows whether broad tasks are under-reading source, tests, or
docs across a repository. That gives the next R&D slice a measured target:
VeriSchema and ReAgent pressure is overwhelmingly source-like, ctxhelm has a
meaningful docs component, and validation pressure is comparatively small.
