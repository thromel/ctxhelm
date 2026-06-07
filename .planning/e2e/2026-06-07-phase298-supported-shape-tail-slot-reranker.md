# Phase 298 - Supported Shape Tail-Slot Reranker

## Scope

Phase 297 proved a gold-backed upper bound: if the gate used
`candidateMissedFileProfilesAt10`, it could recover the remaining corrected
Jina VeriSchema target, `schema_agent/core/state.py`, without target churn. That
oracle is not runtime-promotable because target-miss profiles are eval labels.

Phase 298 replaces that oracle dependency with source-free generated candidate
profiles. Historical commit rows now emit
`supportedSemanticCandidateProfilesAt10`: semantic-generated candidates that did
not land in the selected top 10 but had at least one non-semantic supporting
signal. The eval-only `semantic_supported_shape_tail_slot_reranked` variant
then inserts only candidates matching the measured source-free shape:

- query family: `symbol_identifier`
- path family: `python_source`
- file role: `source`
- support family: `dependency_co_change`

The variant records its contribution in
`supportedShapeTailSlotSemanticRerankerContribution`.

## Pre-Registered Bar

Treat the variant as useful only if it matches the oracle's recovered target
without default-only target churn. A clean result means the Phase 297 upper
bound has a source-free predictor for this fixture slice. A regression means the
shape is too noisy even before broader proof.

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
  > .ctxhelm/e2e/phase298-jina-supported-shape-tail-slot-verischema-older-limit20.json
```

## Result

The source-free shape predictor matches the oracle's clean lift on this slice:

| Field | Value |
| --- | ---: |
| variant | `semantic_supported_shape_tail_slot_reranked` |
| provider status | `eval_only` |
| file Recall@10 | `0.3438596` |
| default target hits | `13` |
| reranked target hits | `14` |
| target-hit delta | `+1` |
| reranker-only target hits | `1` |
| default-only target hits | `0` |
| improved commits | `1` |
| regressed commits | `0` |

The recovered path is:

```text
3507d7c932c4 schema_agent/core/state.py
```

The matching diagnostic is
`semantic_supported_shape_tail_slot_reranker_clean_lift`.

The report also keeps the Phase 297 oracle row, and both contribution summaries
match on this proof: one improved `symbol_identifier` commit, one recovered
`python_source` target, and no default-only target churn.

## Decision

Keep `semantic_supported_shape_tail_slot_reranked` eval-only. It is now
source-free, unlike the oracle, but it is still hand-scoped from one targeted
fixture slice. The next bar is broader range/repo evidence that this exact
supported-candidate shape repeats without target churn before considering any
runtime or default policy.
