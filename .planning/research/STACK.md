# Stack Research: v2.4 Production Semantic & Precision Backends

## Candidate Components

### Local Embedding Backend

Primary candidate: `fastembed` Rust crate.

Why it fits:

- Rust-native integration path.
- ONNX-backed local inference.
- Supported-model discovery.
- Configurable model init.
- Quantization and bring-your-own model surfaces.
- Reranker structs are available for future local reranking experiments.

Adoption shape:

- Add a provider trait behind ctxhelm's existing semantic provider metadata.
- Keep `local_hash` as a deterministic test provider.
- Add a real local provider such as `local_fastembed`.
- Store model id, dimensions, output dtype if available, cache path, provider version, and privacy status.
- Require explicit `ctxhelm semantic build` or equivalent before runtime use.

Source: https://docs.rs/fastembed/

### Vector Store

Preferred v2.4 direction:

- Start with SQLite-backed metadata plus compact vector persistence under ctxhelm storage.
- Keep the search abstraction independent from storage so an HNSW/sqvec/Lance-style index can be swapped later.
- Optimize for deterministic local eval before introducing a heavy external vector server.

Non-goals:

- No Qdrant/Elasticsearch/hosted vector service for v2.4 defaults.
- No cloud vector DB by default.

### Precision Backend

Primary source: SCIP import/generation.

Why it fits:

- Language-agnostic protocol.
- Existing ecosystem of indexers.
- Represents navigation edges useful for callers, definitions, references, and symbol relationships.

Operational shape:

- Keep import path first because ctxhelm already has precision-edge import.
- Add discovery/status for language indexers.
- Add optional generation helpers later in the milestone if safe and deterministic.
- Always report degraded precision status rather than failing context planning.

Sources:

- https://scip-code.org/
- https://sourcegraph.com/docs/code-navigation/precise-code-navigation

### Query Construction

Needed inputs:

- task type
- explicit files/paths/symbols
- error-like strings and stack frames, without storing private logs in eval exports
- commit message/title facets for historical eval
- language/package/test role hints
- active/current-diff anchors

Output:

- lexical query
- semantic query
- symbol query
- graph expansion seed set
- reranker query
- source-free debug trace

### Optional Cloud Providers

Cloud providers should be policy objects, not built-in defaults.

Provider classes:

- Embeddings: OpenAI, Voyage, future provider adapters.
- Reranking: Cohere, Voyage, future local reranker.

Required policy metadata:

- enabled or disabled
- allowed data classes
- source snippets allowed or denied
- git history allowed or denied
- provider/model/version/dimensions
- eval gate that justified use
- rollback target

Sources:

- https://docs.voyageai.com/docs/introduction
- https://docs.cohere.com/docs/rerank-overview
- https://platform.openai.com/docs/models/text-embedding-3-small
