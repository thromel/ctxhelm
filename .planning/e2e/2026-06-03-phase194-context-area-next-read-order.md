# Phase 194: Context Area Next-Read Ordering

## Goal

Phase 193 proved that context-area `nextReadPaths` can recover files missed by
top-10 target-file context, but those paths were still selected by candidate
insertion order inside each area. This phase makes next-read ordering explicit,
deterministic, and source-free.

## Implementation

- Added a private source-free next-read scorer for context areas.
- Unselected context-area candidates now order by:
  - role priority: source/config/schema, then tests, then docs
  - signal priority: anchor/current diff, lexical, symbol, co-change,
    dependency, lexical expansion, memory, semantic, related-test, then lower
    metadata signals
  - weighted signal score
  - candidate confidence
  - original insertion order and path as stable tie-breakers
- Selected target files are unchanged; this only reorders progressive
  `nextReadPaths`.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-compiler context_area_next_reads_order_by_source_free_signal_strength --locked -- --nocapture
cargo test -p ctxhelm-compiler context_areas_include_docs_and_next_read_paths --locked -- --nocapture
cargo test -p ctxhelm-compiler broad_context_area_recall_counts_surfaced_areas --locked -- --nocapture
```

Result: passed.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase194-next-read-order-home \
  /tmp/ctxhelm-rd/phase194-next-read-order-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase194-next-read-order-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase194-next-read-order-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase194-next-read-order-proof.json
```

Result: `releaseGate.decision = promote`.

Next-read recovery delta versus Phase 193:

| Repo | Missed@10 | Phase 193 Recoverable | Phase 194 Recoverable | Top-Pressure Delta | Zero-Selected Delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 1 | 0 | 0 | 0 | 0 |
| ctxhelm | 12 | 9 | 10 | 0 | 0 |
| ReAgent | 0 | 0 | 0 | 0 | 0 |
| VeriSchema | 39 | 10 | 14 | +2 | +3 |

Core retrieval metrics stay unchanged from Phase 193:

| Repo | File Recall@10 | Source Recall@10 | Test Recall@10 | Broad Area Recall |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8` | `1.0` | `1.0` | `0.0` |
| ctxhelm | `0.67777777` | `0.55` | `0.0` | `1.0` |
| ReAgent | `0.8` | `1.0` | `1.0` | `0.0` |
| VeriSchema | `0.35529414` | `0.5277778` | `0.7896825` | `0.5777778` |

## Why It Matters

This improves progressive agent usefulness without spending more top-10 context
budget. The most important measured gain is VeriSchema: source-free ordering
raises next-read recovery from `10 / 39` missed@10 paths to `14 / 39`, including
`8` recoverable paths from top-pressure areas. That makes the broad-area
guidance more likely to point an agent at the right files after the initial
context pack is exhausted.
