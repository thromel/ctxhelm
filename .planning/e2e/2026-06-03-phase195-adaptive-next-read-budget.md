# Phase 195: Adaptive Context Area Next-Read Budget

## Goal

Phase 194 improved `nextReadPaths` ordering, but high-pressure source-like
areas were still capped at four progressive read paths. The latest proof showed
many missed@10 files were already inside surfaced context areas, so the next
useful move is to increase progressive read coverage only where area pressure
justifies it.

## Implementation

- Kept selected target-file ranking unchanged.
- Kept Phase 194 source-free next-read ordering unchanged.
- Added an adaptive `nextReadPaths` cap:
  - default: 4 paths
  - source-like unselected paths >= 6 or validation unselected paths >= 4: 6 paths
  - source-like unselected paths >= 12 or validation unselected paths >= 8: 8 paths
- Added a focused planning test proving high-pressure source areas expose eight
  ordered next-read paths.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-compiler \
  context_area_next_reads_expand_for_high_pressure_source_areas \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  context_area_next_reads_order_by_source_free_signal_strength \
  --locked -- --nocapture
cargo fmt --all -- --check
```

Result: passed.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase195-adaptive-next-read-home \
  /tmp/ctxhelm-rd/phase195-adaptive-next-read-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase195-adaptive-next-read-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase195-adaptive-next-read-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase195-adaptive-next-read-proof.json
```

Result: `releaseGate.decision = promote`.

Selected-file metrics stayed unchanged from Phase 194:

| Repo | File Recall@10 | Source Recall@10 | Test Recall@10 | Effective Validation Recall@10 | Broad Area Recall |
| --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8` | `1.0` | `1.0` | `1.0` | `0.0` |
| ctxhelm | `0.67777777` | `0.55` | `0.0` | `0.0` | `1.0` |
| ReAgent | `0.8` | `1.0` | `1.0` | `1.0` | `0.0` |
| VeriSchema | `0.35529414` | `0.5277778` | `0.7896825` | `1.0` | `0.5777778` |

Next-read recovery delta versus Phase 194:

| Repo | Missed@10 | Phase 194 Recoverable | Phase 195 Recoverable | Top-Pressure Delta | Zero-Selected Delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 1 | 0 | 0 | 0 | 0 |
| ctxhelm | 12 | 10 | 11 | +1 | 0 |
| ReAgent | 0 | 0 | 0 | 0 | 0 |
| VeriSchema | 39 | 14 | 16 | +2 | +1 |

## Why It Matters

This improves the agent-native path without increasing top-10 context. Agents
that exhaust the selected-file pack now get more useful second-step reads only
for areas where source-free pressure shows that four paths are too few. The
largest measured gain remains VeriSchema: adaptive next reads recover `16 / 39`
missed@10 paths versus `14 / 39` in Phase 194, with no selected-file metric
churn.
