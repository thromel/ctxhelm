# Phase 293 - Jina Code Semantic Model Contract

## Scope

Phase 292 rejected more same-query term expansion. Phase 293 tests a different
source-free local semantic path: the explicit `JinaEmbeddingsV2BaseCode`
fastembed model on the same VeriSchema older-range slice, using the best
surviving query mode from Phase 289:

```bash
--semantic-model JinaEmbeddingsV2BaseCode
--semantic-query-mode candidate-path-hints
```

The run first exposed a correctness bug: ctxhelm normalized the Jina code model
to the default `local_fastembed` `384` dimensions, while the model emits
`768`-dimension vectors. Those file vectors were discarded as incompatible, so
the first proof had no semantic candidates. Phase 293 fixes the explicit Jina
model path by:

- normalizing `JinaEmbeddingsV2BaseCode` to `768` dimensions;
- rendering Jina inputs with the fastembed-recommended `query:` and `passage:`
  prefixes;
- including the provider-specific text contract in semantic document vector
  hashes so stale vectors from the old text shape are not reused.

The default `AllMiniLML6V2Q` model and runtime/default semantic behavior remain
unchanged.

## Pre-Registered Bar

Treat this as promotion evidence only if the corrected Jina run improves the
targeted VeriSchema older-range proof without runtime or fusion regressions:

- semantic candidate missed targets fall below Phase 289's `3`;
- selected semantic target hits stay at or above Phase 289's `11`;
- semantic-only non-targets fall or remain controlled;
- `local_semantic` recall improves, or `semantic_tail_slot_reranked` adds
  target hits without regression;
- runtime is not materially worse than the current production-local default.

## Proof Command

```bash
cargo build -p ctxhelm --features local-embeddings --locked

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-model JinaEmbeddingsV2BaseCode \
  --semantic-query-mode candidate-path-hints \
  --format json \
  > .ctxhelm/e2e/phase293-jina-candidate-path-hints-verischema-older-limit20.json
```

## Results

| Mode | Semantic candidate targets | Candidate misses | Selected semantic targets | Semantic-only targets | Semantic-only non-targets | Corroborated target-hit delta | Tail-slot target-hit delta | `local_semantic` runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| AllMini `candidate-path-hints` | `14` | `3` | `11` | `2` | `16` | `-1` | `0` | prior Phase 289 |
| Jina code `candidate-path-hints` | `13` | `1` | `12` | `2` | `12` | `-1` | `0` | `20351ms` |

The corrected Jina code model is a real candidate-quality improvement over the
current AllMini proof on this slice:

- candidate misses fall from `3` to `1`;
- selected semantic target hits rise from `11` to `12`;
- semantic-only non-targets fall from `16` to `12`;
- the only remaining semantic candidate miss is `schema_agent/core/state.py`.

It is not promotion evidence:

- `local_semantic` Recall@10 remains `0.29122806`, equal to `ctxhelm_default`;
- `semantic_corroborated_reranked` still regresses with `targetHitDelta = -1`;
- `semantic_tail_slot_reranked` remains safe but neutral with `targetHitDelta = 0`;
- top-level decision is `hold` because runtime ratio is `6.01x` with recall
  delta `+0.000`.

## Decision

Keep the Jina dimension/text-contract fix for explicit-model correctness and
future targeted diagnostics. Do not promote Jina code as the default
`local_fastembed` model and do not expose it as runtime/default semantic policy.

The result changes the remaining semantic branch: Jina improves candidate
quality enough to be a useful diagnostic backend, but the bottleneck is now
fusion/selection under runtime constraints, not candidate generation alone.
