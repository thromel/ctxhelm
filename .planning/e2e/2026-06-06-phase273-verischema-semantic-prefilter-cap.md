# Phase 273 - VeriSchema Semantic Prefilter Cap Probe

## Scope

Phase 272 showed that `semantic_corroborated_reranked` loses VeriSchema Python
source targets, especially under broad tasks. This phase tests whether the
local-fastembed document prefilter cap is the cause.

This phase changes no runtime behavior.

## Experiment

The default local-fastembed semantic search prefilters source-free semantic
documents before embedding. The probe reran the VeriSchema gate with a larger
prefilter cap:

```bash
cargo build -p ctxhelm --features local-embeddings
CTXHELM_FASTEMBED_DOCUMENT_LIMIT=256 ./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase273-verischema-fastembed-prefilter-256.json
```

Operational note: the first attempt was invalid because a prior featureless
workspace validation had overwritten `./target/debug/ctxhelm`. The recorded
artifact was regenerated immediately after `cargo build -p ctxhelm --features
local-embeddings`.

## Result

| Setting | Default Recall@10 | Local Semantic Recall@10 | Semantic-Corroborated Recall@10 | Semantic candidate target hits | Semantic-only targets | Semantic-only non-targets |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Default cap | `0.39382353` | `0.39382353` | `0.36960787` | `11` | `6` | `17` |
| `CTXHELM_FASTEMBED_DOCUMENT_LIMIT=256` | `0.39382353` | `0.39382353` | `0.36960787` | `11` | `6` | `17` |

The path-family loss is unchanged:

- `scripts +2` route candidate;
- `python_source -10` block regression.

## Decision

Reject "increase local-fastembed prefilter cap" as the next VeriSchema fix.

The Python-source loss is not explained by the default prefilter cap. The next
VeriSchema semantic work should inspect query/document construction, source-free
Python package/module facets, or budget/fusion behavior around Python source
targets rather than simply embedding more documents.
