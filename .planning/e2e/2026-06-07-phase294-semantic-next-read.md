# Phase 294 - Semantic Next-Read Diagnostic

## Scope

Phase 293 showed that corrected Jina candidate-path hints improve VeriSchema
semantic candidate quality, but final Recall@10 and semantic-corroborated
fusion remain held. Phase 294 tests a narrower source-free hypothesis: semantic
may still be useful as bounded next-read guidance after preserving the full
default top K.

This phase adds the eval report field `semanticNextReadContribution`. It does
not create a new ranking variant and does not mutate
`recommended_context_files`. For each commit, it keeps the default top K intact,
then measures up to two semantic-ranked files that are not already in that
default top K.

## Pre-Registered Bar

Treat this only as diagnostic guidance unless semantic appended next reads show
meaningful target recovery with controlled non-target noise. Any result with
only non-target appended paths rejects next-read promotion and keeps semantic
out of runtime/default policy.

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
  > .ctxhelm/e2e/phase294-jina-semantic-next-read-verischema-older-limit20.json
```

## Result

`semanticNextReadContribution` on the targeted VeriSchema older-range proof:

| Field | Value |
| --- | ---: |
| evaluated commits | `19` |
| append limit per commit | `2` |
| commits with appended files | `1` |
| commits with target hits | `0` |
| commits with only non-targets | `1` |
| appended files | `1` |
| appended target hits | `0` |
| appended non-targets | `1` |
| appended target hit rate | `0.0` |

The only appended next-read path is a non-target:

```text
cd8e51daefcd tests/core/test_state_validator.py
```

The matching diagnostic is `semantic_next_read_noise_hold`.

## Decision

Keep `semanticNextReadContribution` as a source-free eval diagnostic. It is
useful because it distinguishes "semantic found target candidates but top-K
selection dropped them" from "semantic has no useful post-top-K next-read path
once the default top K is protected."

Do not promote semantic next-read guidance from this result. On the targeted
Jina candidate-path proof, preserving the default top K leaves only one bounded
semantic append and it is noise.
