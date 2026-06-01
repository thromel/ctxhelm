# Feature Research: v2.4 Production Semantic & Precision Backends

## Feature Direction

v2.4 should ship production retrieval capability, but with strong proof boundaries:

- stronger local embeddings
- precision-enriched semantic documents
- better query construction
- optional policy-gated providers
- eval gates that decide whether semantic/precision should be recommended

## User Stories

1. As a maintainer, I can build a local semantic index using a real embedding backend and inspect model/provider metadata without uploading source code.
2. As a maintainer, I can see whether precision inputs are available, stale, missing, or degraded before trusting precision-expanded retrieval.
3. As an agent, I can call `prepare_task` and receive candidates whose semantic/precision reasons explain what signal selected them.
4. As a maintainer, I can run a paired ablation showing whether semantic, precision, and reranking improved Recall@10, MRR, runtime, and known failure families.
5. As a privacy-sensitive user, I can keep cloud embeddings and cloud reranking disabled unless a repo policy explicitly enables them.
6. As a product owner, I can prevent semantic defaults from shipping when the fixed corpus shows neutral or regressive results.

## Feature Boundaries

In scope:

- local real embedding provider
- source-free vector metadata and status
- semantic document builder
- SCIP/LSP precision input status and semantic enrichment
- query facet construction
- optional provider policy scaffolding
- eval gates for promotion/rollback

Out of scope:

- default cloud embeddings
- default cloud reranking
- hosted vector DB
- broad autonomous indexer installation
- agent-native deep integration work beyond additive CLI/MCP contracts
- desktop inspector changes

## Product Proof Requirement

The milestone is only a win if reports can say one of these clearly:

- "local semantic/precision improves fixed-corpus retrieval enough to recommend it for this repo class"
- "semantic remains neutral/regressive, so ctxhelm keeps lexical/graph/history defaults and preserves the backend as opt-in"

Both outcomes are acceptable. Quietly enabling a neutral semantic path is not.
