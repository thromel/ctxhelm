# Phase 81 Warm Cache Latency Proof

## Goal

Make eval-cache runtime evidence trustworthy enough for production-readiness
decisions. Before this phase, a cache hit incremented `cacheHits`, but retained
the cold run's `totalMillis`, `commitMillis`, and slow-commit diagnostics. That
made warm-cache proof output misleading.

## Change

- Cache hits now report the warm cache lookup runtime.
- Cached reports set `commitMillis`, ranking time, pack/compiler time, git
  sample time, and slow commits to zero for the warm lookup.
- Cached reports preserve the cached quality metrics and source-free proof
  fields.
- Added a focused unit test for warm cache runtime semantics.

## Proof Config

Committed config:

- `.planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json`

The config enables `cacheEnabled: true` and `forceRefresh: false` across the
four-repo fixed corpus.

## Cold Run

Command:

```bash
rm -rf /tmp/ctxhelm-phase81-cache && mkdir -p /tmp/ctxhelm-phase81-cache
CTXHELM_HOME=/tmp/ctxhelm-phase81-cache cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json \
  --format json > /tmp/ctxhelm-phase81-cold-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase81-cold-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase81-warm-cache-cold-proof.json`

Gate decision: `promote`.

## Warm Run

Command:

```bash
CTXHELM_HOME=/tmp/ctxhelm-phase81-cache cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json \
  --format json > /tmp/ctxhelm-phase81-warm-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase81-warm-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase81-warm-cache-warm-proof.json`

Gate decision: `promote`.

| Corpus | Cold runtime ms | Warm runtime ms | Warm cache hits | Warm cache misses | Warm commit ms | Warm slow commits |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 8571 | 1 | 1 | 0 | 0 | 0 |
| ctxhelm | 8097 | 1 | 1 | 0 | 0 | 0 |
| ReAgent | 4986 | 1 | 1 | 0 | 0 | 0 |
| VeriSchema | 6560 | 1 | 1 | 0 | 0 | 0 |

## Quality Check

The warm proof preserves the Phase 80 quality behavior:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Effective validation recall | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `match` | 1.000 | 1.000 | 1.000 | 0.000 |
| ctxhelm | `beat` | 0.444 | 0.306 | 0.000 | 0.000 |
| ReAgent | `beat` | 1.000 | 0.571 | 1.000 | 0.000 |
| VeriSchema | `beat` | 0.205 | 0.082 | 1.000 | 0.000 |

## Validation

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler cached_eval_runtime_reports_warm_lookup_cost -- --nocapture
cargo run -p ctxhelm -- eval proof --config .planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase81-cold-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase81-warm-proof.json
```
