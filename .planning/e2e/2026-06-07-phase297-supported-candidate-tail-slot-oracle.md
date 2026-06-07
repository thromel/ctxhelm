# Phase 297 - Supported Candidate Tail-Slot Oracle

## Scope

Phase 296 showed corrected Jina's remaining VeriSchema candidate-path miss is
`schema_agent/core/state.py` with `dependency_co_change` support in the
semantic candidate-missed profile. Phase 297 measures the upper bound of that
surface: if an eval-only oracle inserts semantic-generated missed targets with
non-semantic support into protected tail slots, does the candidate recover
without target churn?

This phase adds `semantic_supported_candidate_tail_slot_oracle` to semantic gate
reports and records its contribution in
`supportedCandidateTailSlotRerankerContribution`. The variant is deliberately
not runtime-promotable because it consumes `candidateMissedFileProfilesAt10`,
which are built from eval target misses.

## Pre-Registered Bar

Treat the variant as useful only if it adds target hits without default-only
target churn. A clean result proves there is a supported fusion/top-K ordering
surface to learn from. A regression means even the gold-backed supported
candidate surface is unsafe under the current protected tail-slot budget.

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
  > .ctxhelm/e2e/phase297-jina-supported-candidate-tail-slot-oracle-verischema-older-limit20.json
```

## Result

The oracle is a clean upper-bound lift:

| Field | Value |
| --- | ---: |
| variant | `semantic_supported_candidate_tail_slot_oracle` |
| file Recall@10 | `0.3438596` |
| default target hits | `13` |
| oracle target hits | `14` |
| target-hit delta | `+1` |
| reranker-only target hits | `1` |
| default-only target hits | `0` |
| improved commits | `1` |
| regressed commits | `0` |

The recovered path is:

```text
3507d7c932c4 schema_agent/core/state.py
```

The contribution localizes the lift to:

- query family: `symbol_identifier`
- path family: `python_source`
- support profile from Phase 296: `dependency_co_change`

The matching diagnostic is
`semantic_supported_candidate_tail_slot_oracle_clean_lift`.

## Decision

Keep the oracle eval-only. It is gold-backed and cannot be exposed as runtime
policy. The result does prove that corrected Jina's supported missed-candidate
surface can recover a VeriSchema target under protected tail-slot insertion
without displacing default targets.

The next semantic R&D step should replace the oracle dependency with a
source-free predictor for supported semantic candidate misses, scoped first to
the `symbol_identifier` / `python_source` shape and guarded by no default-only
target churn.
