# Phase 274 - VeriSchema Python Module Facet Rejection

## Scope

Phase 273 showed that VeriSchema's Python-source semantic gap is not fixed by a
larger local-fastembed prefilter cap. This phase tests a targeted
query/document-construction idea: add source-free Python package/module facets
derived only from file paths.

The implementation was evaluated and then reverted because it worsened the
measured gate.

## Experiment

Temporary semantic document facets:

- `path_namespace`
- `python_package`
- `python_dotted_module`
- `python_module_aliases`

These were path-derived only and did not read source text.

Validation before the gate:

```bash
cargo fmt --check
cargo test -p ctxhelm-index semantic_documents_include_source_free_python_module_facets --locked
cargo test -p ctxhelm-index semantic_documents_include_source_free_identifier_alias_facets --locked
cargo build -p ctxhelm --features local-embeddings
```

VeriSchema gate:

```bash
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase274-verischema-python-module-facets.json
```

## Result

| Variant | Baseline Phase 272 | Python module facets |
| --- | ---: | ---: |
| Default Recall@10 | `0.39382353` | `0.39382353` |
| Local Semantic Recall@10 | `0.39382353` | `0.39382353` |
| Semantic-Corroborated Recall@10 | `0.36960787` | `0.32294118` |
| Semantic candidate target hits | `11` | `12` |
| Semantic candidate missed targets | `0` | `2` |
| Semantic-only target hits | `6` | `5` |
| Semantic-only non-targets | `17` | `18` |

Path-family movement also worsened:

- `python_source` moved from `-10 block` to `-8 churn`;
- `docs -1 block` appeared;
- only one Python source reranker-only target was added, while nine Python
  source default-only targets remained.

## Decision

Reject path-derived Python package/module facets for semantic search documents.

The facets are source-free, but they increase semantic-corroborated churn and
lower recall. The temporary code was reverted. Future VeriSchema work should
target better task/query construction or fusion constraints around Python source
targets rather than broadening source-free Python path metadata.
