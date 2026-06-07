# Phase 291 - Candidate-Sibling Semantic Query Hints

## Scope

Phase 290 removed the known semantic-corroborated regression with a conservative
tail-slot selector, but it did not add target hits. The remaining VeriSchema
semantic misses still included source-free path-neighborhood cases such as:

- `tests/nlp/test_transformer_ner.py`, mirrored from selected NLP source paths;
- `schema_agent/core/state.py`, near selected `schema_agent/core/*` support
  files;
- workflow, gate, prompt, and verification support files.

Phase 291 adds an eval-only semantic query construction probe:

```bash
--semantic-query-mode candidate-sibling-path-hints
```

The mode includes the existing bounded candidate-path aliases, then adds at
most four source-free aliases from same-directory or mirrored-test inventory
paths near top lexical candidates. Runtime/default query behavior remains
`plain`.

## Pre-Registered Bar

Treat this as useful query-construction evidence only if it improves the
targeted VeriSchema older-range semantic candidate generation without worsening
selection:

- candidate missed targets fall below Phase 289's `3`;
- selected semantic target hits stay at or above Phase 289's `11`;
- semantic-only non-targets do not materially rise;
- `local_semantic`, `semantic_corroborated_reranked`, or
  `semantic_tail_slot_reranked` recall improves without new regressions.

## Proof Command

```bash
cargo build -p ctxhelm --features local-embeddings --locked

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-query-mode candidate-sibling-path-hints \
  --format json \
  > .ctxhelm/e2e/phase291-candidate-sibling-path-hints-verischema-older-limit20.json
```

## Results

| Mode | Semantic candidate targets | Candidate misses | Selected semantic targets | Semantic-only targets | Semantic-only non-targets | Corroborated target-hit delta | Tail-slot target-hit delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `candidate-path-hints` | `14` | `3` | `11` | `2` | `16` | `-1` | `0` |
| `candidate-sibling-path-hints` | `14` | `4` | `10` | `2` | `15` | `-1` | `0` |

The sibling hints add one candidate-generation miss instead of removing the
existing misses. The missed candidate list includes
`tests/nlp/test_transformer_ner.py`, `schema_agent/core/state.py`, and
`schema_agent/agents/repair_agent.py`. The `local_semantic` and tail-slot
variants remain at Recall@10 `0.29122806`; `semantic_corroborated_reranked`
still regresses to `0.28245613`.

## Decision

Reject candidate-sibling path hints as the next semantic promotion path. The
source-free path-neighborhood hypothesis is measurable, but it worsens candidate
generation/selection compared with Phase 289 and leaves the same corroborated
regression.

The next semantic branch should move away from adding more path aliases to a
single local-fastembed query. Better candidates are a richer but still
source-free document/query construction for prompt/workflow/verification
concepts, or a narrower learned separator with held-out no-regress evidence.
