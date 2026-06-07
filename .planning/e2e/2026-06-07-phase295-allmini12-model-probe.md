# Phase 295 - AllMiniLML12 Model Probe

## Scope

Phase 293 showed corrected Jina candidate-path hints improved VeriSchema
semantic candidate quality but not Recall@10 or fusion safety. Phase 295 checks
the cheaper alternate local model branch already accepted by the CLI mapping:
`AllMiniLML12V2Q` and `AllMiniLML12V2` with the same bounded
`candidate-path-hints` query mode.

## Pre-Registered Bar

Treat a 12-layer MiniLM variant as useful diagnostic evidence only if it creates
semantic candidate targets on the targeted VeriSchema older-range slice and
matches or improves the current Jina candidate-quality counters without adding
new top-K regressions. A no-candidate result rejects this model branch.

## Proof Commands

```bash
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

| Model | Decision | Local semantic Recall@10 | Semantic candidate target hits | Semantic-only target hits | Semantic-only non-targets | Next-read appended files | Corroborated target-hit delta | Tail-slot target-hit delta |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `AllMiniLML12V2Q` | `hold` | `0.29122806` | `0` | `0` | `0` | `0` | `-1` | `0` |
| `AllMiniLML12V2` | `hold` | `0.29122806` | `0` | `0` | `0` | `0` | `-1` | `0` |

Both variants emit `semantic_contribution_no_candidates` for all `19`
evaluated commits. The current gate still reports the known
semantic-corroborated displacement/regression around
`schema_agent/prompts/normalization.py`, while the tail-slot variant remains
safe but neutral.

## Decision

Reject the AllMiniLML12 model branch for the current VeriSchema candidate-path
setup. Compared with corrected Jina, the 12-layer MiniLM variants do not improve
candidate generation; they produce no semantic candidates at all in this proof.

The remaining model/document path should not spend more time on documented
MiniLM12 variants as a simple swap. Jina remains the better diagnostic backend
for this slice, but still not runtime/default policy.
