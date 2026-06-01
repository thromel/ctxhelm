# Architecture Research: v2.4 Production Semantic & Precision Backends

## Proposed Architecture

```text
Task / commit / active context
  -> query facet builder
  -> lexical + graph + history + tests
  -> semantic query
  -> local embedding provider
  -> vector candidate store
  -> precision-enriched graph expansion
  -> optional reranker/provider gate
  -> paired eval gate
  -> context compiler
```

## New Internal Surfaces

### SemanticDocument

Typed document used for embedding and reranking.

Fields:

- id
- kind: file, symbol, test, doc, commit, memory
- path
- symbol
- role
- language
- text_hash
- source_policy
- semantic_text
- included_facets
- precision_status
- token_estimate

The stored eval/export form must be source-free. The runtime text can be materialized from safe inventory when building local embeddings.

### EmbeddingProvider

Provider trait:

- provider_id
- model_id
- dimensions
- supports_local_only
- supports_batch
- max_input_tokens
- embed_documents
- embed_query
- privacy_status
- health/status

Providers:

- `local_hash`: deterministic test provider, never quality default
- `local_fastembed`: first real local backend
- future optional cloud providers under policy gates

### PrecisionInputStatus

Status object:

- unavailable
- present
- stale
- failed
- partial
- degraded

Every context plan using precision should expose the status and avoid hiding fallback behavior.

### RetrievalGate

Gate object:

- corpus id
- variant id
- baseline variant
- Recall@10 delta
- MRR@10 delta
- test recall delta
- runtime delta
- token ROI delta
- known-miss deltas
- verdict: promote, keep opt-in, block, insufficient evidence

## Data Flow

1. Build semantic documents from safe inventory, symbols, tests, docs, cards, and optional precision edges.
2. Embed documents locally by default.
3. Persist source-free vector metadata and local vector files under ctxhelm storage.
4. At query time, construct lexical, semantic, and reranker queries from the same task facets.
5. Fuse semantic candidates with existing lexical/graph/history/test candidates.
6. Run paired evals before changing default policy.
7. Render provider and gate metadata through CLI, MCP, docs, and product proof.

## Architecture Constraints

- No source text in eval reports, feature exports, or proof artifacts.
- No cloud provider without explicit repo policy.
- No mandatory SCIP/LSP generator for normal `prepare_task`.
- No MCP tool surface expansion unless a new tool is unavoidable; prefer additive fields and existing `search`, `prepare_task`, `get_pack`, and `eval` surfaces.
