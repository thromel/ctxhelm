# Phase 296 - Semantic Candidate-Missed Support Profiles

## Scope

Phase 293 showed corrected Jina candidate-path hints improve VeriSchema
semantic candidate quality, Phase 294 rejected semantic as bounded next-read
guidance, and Phase 295 rejected MiniLM12 variants as simple model swaps. Phase
296 narrows the remaining Jina question: when semantic generates a retrieval
target that final top-K selection drops, does that missed semantic candidate
also have non-semantic source-free support?

This phase adds `semanticCandidateMissedSupportProfiles` to
`semanticContribution` and `candidateMissedSupportProfiles` to each
`semanticContribution.queryFamilyContributions[]` entry. The support family is
derived from the missed-candidate profile signals, excluding `semantic`, with a
fallback to selected non-semantic signal rankings for older or sparse reports.

## Pre-Registered Bar

Treat this as diagnostic routing evidence only. A supported semantic candidate
miss points to fusion/top-K ordering work before document or model expansion.
Unsupported semantic candidate misses point to query/document coverage after
supported fusion misses are exhausted. Neither case promotes semantic retrieval
to runtime/default policy without recall lift and regression safety.

## Proof Commands

```bash
cargo build -p ctxhelm --features local-embeddings --locked

./target/debug/ctxhelm semantic status \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --semantic-provider local_fastembed \
  --semantic-model JinaEmbeddingsV2BaseCode \
  --format json \
  > .ctxhelm/e2e/phase296-jina-semantic-status.json

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-model JinaEmbeddingsV2BaseCode \
  --semantic-query-mode candidate-path-hints \
  --format json \
  > .ctxhelm/e2e/phase296-jina-candidate-missed-support-verischema-older-limit20.json
```

Provider preflight confirmed `providerAvailable = true`, `providerStatus =
available`, `providerRole = production_local`, `qualityBackend = true`, and
`privacyStatus.localOnly = true`.

## Result

The corrected Jina candidate-path proof remains held:

| Field | Value |
| --- | ---: |
| evaluated commits | `19` |
| decision | `hold` |
| decision reason | `Held: local semantic recall delta +0.000, precision delta +0.000; keep opt-in.` |
| semantic candidate target hits | `13` |
| semantic candidate missed targets | `1` |
| semantic-only target hits | `2` |
| semantic-only non-targets | `12` |
| next-read target/non-target appends | `0 / 1` |

The one semantic-generated missed target is `schema_agent/core/state.py`, and
its candidate-missed support profile is:

```json
{
  "supportFamily": "dependency_co_change",
  "candidateMissedTargetCount": 1,
  "examplePaths": ["schema_agent/core/state.py"]
}
```

The matching diagnostics are `semantic_candidate_fusion_gap` and
`semantic_candidate_fusion_supported_gap`.

## Decision

Keep corrected Jina as diagnostic-only. Phase 296 confirms the remaining
candidate miss is not a broad semantic document/model generation failure on
this slice: semantic generated `schema_agent/core/state.py`, and that candidate
also had dependency/co-change support, but final top-K selection still dropped
it.

The next semantic R&D slice should target fusion/top-K ordering for
semantic-generated missed targets with non-semantic support. Do not spend the
next slice on broader semantic documents, MiniLM12 model swaps, or semantic
next-read promotion unless a new proof shows this supported fusion gap is gone.
