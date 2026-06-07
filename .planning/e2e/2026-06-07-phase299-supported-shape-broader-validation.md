# Phase 299 - Supported Shape Broader Validation

## Scope

Phase 298 added the eval-only source-free
`semantic_supported_shape_tail_slot_reranked` variant for the measured
`symbol_identifier` / `python_source` / `dependency_co_change` supported
semantic candidate shape. Phase 299 tests whether that shape repeats across the
stable older/recent ranges for RefactoringMiner, ctxhelm, ReAgent, and
VeriSchema.

The sweep was run after a clean feature-enabled rebuild:

```bash
cargo build -p ctxhelm --features local-embeddings --locked
```

Each gate used:

```bash
./target/debug/ctxhelm eval gate \
  --repo <fixture> \
  --base <range-base> --head <range-head> \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-model JinaEmbeddingsV2BaseCode \
  --semantic-query-mode candidate-path-hints \
  --format json
```

## Result

| Slice | Commits | Shape delta | Improved | Regressed | Default-only targets | Path |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner older | 20 | `0` | 0 | 0 | 0 |  |
| RefactoringMiner recent | 20 | `0` | 0 | 0 | 0 |  |
| ctxhelm older | 20 | `0` | 0 | 0 | 0 |  |
| ctxhelm recent | 20 | `0` | 0 | 0 | 0 |  |
| ReAgent older | 20 | `0` | 0 | 0 | 0 |  |
| ReAgent recent | 20 | `0` | 0 | 0 | 0 |  |
| VeriSchema older | 19 | `+1` | 1 | 0 | 0 | `schema_agent/core/state.py` |
| VeriSchema recent | 20 | `0` | 0 | 0 | 0 |  |

Aggregate:

- evaluated commits: `159`
- target-hit delta: `+1`
- improved commits: `1`
- regressed commits: `0`
- default-only target hits: `0`

The only non-neutral contribution remains the original VeriSchema older
`symbol_identifier` case:

```text
3507d7c932c4 schema_agent/core/state.py
```

## Decision

Keep `semantic_supported_shape_tail_slot_reranked` eval-only. The broader sweep
is safe but too sparse for runtime or default promotion: it proves the
source-free shape does not churn targets on these eight slices, but it does not
show repeated lift outside the original VeriSchema older case.

The next useful semantic work should either find a broader source-free
candidate-retention signal for supported semantic candidates, or build a
held-out learned separator with repeated no-churn applications. Do not promote
Jina, the supported-shape variant, or semantic tail-slot insertion by default
from this evidence.
