# Phase 295 - AllMiniLML12 Model Probe

## Scope

Phase 293 showed corrected Jina candidate-path hints improved VeriSchema
semantic candidate quality but not Recall@10 or fusion safety. Phase 295 checks
the cheaper alternate local model branch already accepted by the CLI mapping:
`AllMiniLML12V2Q` and `AllMiniLML12V2` with the same bounded
`candidate-path-hints` query mode.

The proof requires a binary built with `--features local-embeddings`. An
earlier non-feature local binary returned a misleading no-candidate result, so
the accepted evidence below is from the feature-enabled rerun after confirming
`providerAvailable = true` for `local_fastembed`.

## Pre-Registered Bar

Treat a 12-layer MiniLM variant as useful diagnostic evidence only if it matches
or improves the current Jina candidate-quality counters without adding new
top-K regressions or worse next-read noise. A no-candidate result, a worse
candidate-miss result, or higher noise rejects this model branch.

## Proof Commands

```bash
cargo build -p ctxhelm --features local-embeddings --locked

./target/debug/ctxhelm semantic status \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --semantic-provider local_fastembed \
  --semantic-model JinaEmbeddingsV2BaseCode \
  --format json

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-model AllMiniLML12V2Q \
  --semantic-query-mode candidate-path-hints \
  --format json \
  > .ctxhelm/e2e/phase295-allmini12q-candidate-path-hints-verischema-older-limit20.json

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-model AllMiniLML12V2 \
  --semantic-query-mode candidate-path-hints \
  --format json \
  > .ctxhelm/e2e/phase295-allmini12-candidate-path-hints-verischema-older-limit20.json
```

## Results

| Model | Decision | Local semantic Recall@10 | Candidate targets | Candidate misses | Semantic-only targets | Semantic-only non-targets | Next-read target/non-target appends | Runtime ratio | Corroborated target-hit delta | Tail-slot target-hit delta |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `AllMiniLML12V2Q` | `hold` | `0.29122806` | `14` | `3` | `2` | `18` | `0 / 3` | `6.75x` | `-1` | `0` |
| `AllMiniLML12V2` | `hold` | `0.29122806` | `13` | `3` | `2` | `19` | `0 / 3` | `7.34x` | `-1` | `0` |

Both variants are worse than corrected Jina on the targeted slice:

- Jina candidate misses: `1`; MiniLM12 candidate misses: `3`.
- Jina semantic-only non-targets: `12`; MiniLM12 variants: `18` and `19`.
- Jina next-read appends: `0` target / `1` non-target; MiniLM12 variants:
  `0` target / `3` non-targets.
- Both MiniLM12 variants keep the known semantic-corroborated
  displacement/regression around `schema_agent/prompts/normalization.py`.
- Tail-slot reranking remains safe but neutral.

## Decision

Reject the AllMiniLML12 model branch for the current VeriSchema candidate-path
setup. Compared with corrected Jina, the 12-layer MiniLM variants do not improve
candidate generation, add more semantic-only non-targets, add more next-read
noise, and retain the same unsafe semantic-corroborated fusion behavior.

The remaining model/document path should not spend more time on documented
MiniLM12 variants as a simple swap. Jina remains the better diagnostic backend
for this slice, but still not runtime/default policy.
