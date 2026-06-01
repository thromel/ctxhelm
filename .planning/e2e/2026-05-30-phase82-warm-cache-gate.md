# Phase 82 Warm Cache Release Gate

## Goal

Make warm-cache latency evidence enforceable in the product proof gate. Phase
81 made cache-hit runtime reporting accurate; Phase 82 prevents future
regressions where cached proof reports carry stale cold-run timings or exceed a
bounded steady-state lookup threshold.

## Change

- Product proof release decisions now block cache-hit reports when:
  - cache misses are mixed into a cache-hit proof,
  - warm lookup runtime exceeds `1000ms`,
  - commit-loop, git sample, ranking, or pack/compiler timings are non-zero,
  - slow-commit diagnostics are still present.
- Product proof corpus notes now include warm-cache hit evidence.
- Added focused release-gate tests for stale warm-cache runtime, slow warm
  lookup runtime, and fast warm-cache promotion.

## Proof

Command:

```bash
rm -rf /tmp/ctxhelm-phase82-cache && mkdir -p /tmp/ctxhelm-phase82-cache
CTXHELM_HOME=/tmp/ctxhelm-phase82-cache cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json \
  --format json > /tmp/ctxhelm-phase82-warm-gate-cold-proof.json
CTXHELM_HOME=/tmp/ctxhelm-phase82-cache cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json \
  --format json > /tmp/ctxhelm-phase82-warm-gate-warm-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase82-warm-gate-cold-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase82-warm-gate-warm-proof.json
```

Committed artifacts:

- `.ctxhelm/e2e/phase82-warm-cache-gate-cold-proof.json`
- `.ctxhelm/e2e/phase82-warm-cache-gate-warm-proof.json`

Both gates promote.

| Corpus | Warm runtime ms | Cache hits | Cache misses | Warm commit ms | Warm slow commits |
| --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 1 | 1 | 0 | 0 | 0 |
| ctxhelm | 1 | 1 | 0 | 0 | 0 |
| ReAgent | 1 | 1 | 0 | 0 | 0 |
| VeriSchema | 1 | 1 | 0 | 0 | 0 |

Every warm corpus verdict includes a source-free note of the form
`warm cache proof hit 1 cached report(s) in 1ms`.

## Validation

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler warm_cache -- --nocapture
python3 scripts/check-product-proof.py .ctxhelm/e2e/phase82-warm-cache-gate-cold-proof.json
python3 scripts/check-product-proof.py .ctxhelm/e2e/phase82-warm-cache-gate-warm-proof.json
```
