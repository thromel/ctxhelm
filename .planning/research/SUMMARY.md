# Research Summary: v2.4 Production Semantic & Precision Backends

## Research Question

How should ctxpack turn semantic and precision retrieval from local scaffolding into a production-quality, measured retrieval improvement while preserving the local-first, read-only, source-safe product contract?

## Current Evidence

The May 19 RefactoringMiner semantic ablation is the controlling project evidence:

- Default File Recall@10: 0.518631
- Semantic-enabled File Recall@10: 0.518631
- Default MRR@10: 0.6375
- Semantic-enabled MRR@10: 0.6350
- Runtime increased from 84,398 ms to 104,901 ms
- Semantic-only changed-file hits: 0/82

Conclusion: the current `local_hash` provider is useful as a privacy-safe contract and eval scaffold, but it is not a quality backend. v2.4 must not enable semantic defaults until stronger backends and richer semantic documents pass fixed-corpus gates.

## Primary Findings

### 1. Local embeddings should use a real model backend, not hashes

`fastembed` is a practical Rust-first candidate because it exposes `TextEmbedding`, supported model listing, configurable init options, ONNX sources, quantization modes, and bring-your-own embedding/reranker model structs. That matches ctxpack's local-first architecture and lets the first production backend remain in-process and source-safe.

Source: https://docs.rs/fastembed/

### 2. Semantic text quality is as important as the embedding model

Embedding raw file names or weak snippets will not beat lexical search on code. v2.4 should construct semantic documents from source-free and safe-source-derived structure:

- path and package role
- symbol names and signatures
- imports/exports
- test names and fixture names
- precision callers/callees when available
- docs/card summaries where source-safe
- recent commit/task facets without storing private prompt text

Tree-sitter remains the broad parsing layer because it exposes syntax node types and source positions. SCIP/LSP should enrich semantic documents and graph edges where available, not replace Tree-sitter.

Sources:

- https://tree-sitter.github.io/tree-sitter/using-parsers/2-basic-parsing.html
- https://microsoft.github.io/language-server-protocol/
- https://scip-code.org/

### 3. SCIP should be an optional precision upgrade with degraded-mode reporting

SCIP is a language-agnostic protocol for code navigation data such as go-to-definition and find-references. Sourcegraph's docs also show the operational reality: precise navigation is opt-in, requires indexes, and search-based navigation is the fallback when precise indexes are unavailable.

ctxpack should mirror that: import or generate SCIP/LSP-derived edges when available, report missing/stale/degraded precision inputs, and fall back to lexical, Tree-sitter, graph, history, and test signals without failing the main workflow.

Sources:

- https://sourcegraph.com/docs/code-navigation/precise-code-navigation
- https://scip-code.org/

### 4. Cloud embeddings and reranking are valuable but must stay policy-gated

Voyage and Cohere both describe the standard retrieval stack: embeddings produce vectors for semantic retrieval, rerankers score query-document relevance after first-stage retrieval, and reranking can refine BM25/vector candidates. OpenAI's current embedding docs also position embeddings as a relatedness/search primitive.

For ctxpack, this supports optional provider interfaces, not default cloud use. Every cloud provider must require explicit repo policy, disclose source-sharing implications, record provider/model/dimension/version metadata, and pass paired eval gates before becoming recommended.

Sources:

- https://docs.voyageai.com/docs/introduction
- https://docs.cohere.com/docs/rerank-overview
- https://platform.openai.com/docs/models/text-embedding-3-small

### 5. v2.4 needs quality gates, not another retrieval checkbox

The product risk is adding an expensive semantic path that changes candidate sets without improving gold-file recall or agent behavior. v2.4 should require:

- paired default vs semantic vs precision vs reranker comparisons on the same fixed corpus
- Recall@10 and MRR lift thresholds
- runtime and cache-hit budgets
- failure-family deltas for known misses
- provider rollback metadata
- explicit "do not promote" verdicts when semantic is neutral or regressive

## Recommended Architecture Decision

Build v2.4 in this order:

1. Production local embedding backend contract and vector store.
2. Precision-enriched semantic document builder.
3. Query construction pipeline for tasks, commits, paths, symbols, and errors.
4. Provider/reranker policy gates.
5. Fixed-corpus eval gates and release proof.

Do not enable semantic by default as part of this milestone unless the eval gate proves measurable lift.

## Open Risks

- Local ONNX inference can add binary/download/runtime complexity.
- Code-specialized cloud models may outperform local defaults but conflict with the trust contract.
- SCIP automation can be fragile because language indexers depend on project setup.
- Better embeddings may still fail if semantic documents are weak or if benchmark tasks are primarily exact-identifier tasks.
- Eval lift on RefactoringMiner may not generalize to TS/Python repos without additional corpora.
