# Phase 193: Context Area Next-Read Recovery

## Goal

Phase 192 showed where broad context-area inspection pressure remains, but it
did not prove whether the progressive `nextReadPaths` are actually useful after
top-10 target-file budget is exhausted. This phase adds a source-free recovery
metric that answers a concrete question:

Can the context-area next-read guidance recover files missed by top-10 context?

## Implementation

- Added `contextAreaNextReadSummary` to `HistoricalEvalReport`.
- Aggregated:
  - `missedFileCountAt10`
  - `nextReadRecoverableCount`
  - `topPressureNextReadRecoverableCount`
  - `zeroSelectedAreaRecoverableCount`
  - `sourceTextLogged`
- Rendered the summary in historical eval markdown and benchmark-suite markdown.
- Extended `scripts/check-product-proof.py` to validate summary arithmetic and
  source-free status.

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
rm -rf /tmp/ctxhelm-phase193-next-read-recovery-home \
  /tmp/ctxhelm-rd/phase193-next-read-recovery-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase193-next-read-recovery-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase193-next-read-recovery-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase193-next-read-recovery-proof.json
```

Result: `releaseGate.decision = promote`.

Next-read recovery from the proof:

| Repo | Missed@10 | Next-Read Recoverable | Top-Pressure Recoverable | Zero-Selected Recoverable |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | 1 | 0 | 0 | 0 |
| ctxhelm | 12 | 9 | 3 | 4 |
| ReAgent | 0 | 0 | 0 | 0 |
| VeriSchema | 39 | 10 | 6 | 4 |

Core retrieval metrics stay unchanged from Phase 192:

| Repo | File Recall@10 | Source Recall@10 | Test Recall@10 | Broad Area Recall |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8` | `1.0` | `1.0` | `0.0` |
| ctxhelm | `0.67777777` | `0.55` | `0.0` | `1.0` |
| ReAgent | `0.8` | `1.0` | `1.0` | `0.0` |
| VeriSchema | `0.35529414` | `0.5277778` | `0.7896825` | `0.5777778` |

## Why It Matters

This separates two product claims that were previously blended:

- top-10 selected-file recall
- progressive agent read usefulness after top-10 budget is exhausted

The result shows that context-area guidance is not just decorative for broad
tasks. On ctxhelm, `9 / 12` top-10 misses are recoverable through source-free
next-read paths. On VeriSchema, `10 / 39` are recoverable, with `6` recoverable
from top-pressure areas. The remaining VeriSchema miss pressure is now a sharper
ranking target: improve next-read ordering and area representative selection
before increasing top-10 budget.
