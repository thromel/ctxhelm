# Phase 56 Research: Production Local Semantic Backend

**Phase:** 56 - Production Local Semantic Backend
**Status:** Complete
**Research question:** What does the executor need to know to replace the current semantic scaffold with a production local embedding backend while preserving ctxhelm's source-safe, local-first contract?

## Phase Boundary

Phase 56 covers the backend/provider foundation only:

- real local embedding provider contract and implementation path
- source-free vector/provider metadata persistence
- provider status and diagnostics
- clear `local_hash` scaffold labeling
- CLI/MCP-compatible semantic candidate provenance

Phase 56 does not build precision-enriched semantic documents, reranking, cloud provider execution, or default-promotion gates. Those are later v2.4 phases.

## Current Code Surface

### Existing Semantic Provider

`crates/ctxhelm-index/src/semantic.rs` currently owns:

- `SemanticProviderConfig`
- `SemanticOptions`
- `SemanticSearchResult`
- `SemanticSearchReport`
- `SemanticVectorRecord`
- `semantic_search_report`
- `semantic_vector_records`
- `sync_semantic_index_to_store`
- `vectorize_text`

The current provider constants are:

- `DEFAULT_SEMANTIC_PROVIDER = "local_hash"`
- `DEFAULT_SEMANTIC_MODEL = "ctxhelm-local-hash-v1"`
- `DEFAULT_SEMANTIC_DIMENSIONS = 64`
- `DEFAULT_SEMANTIC_DISTANCE = "cosine"`

This is deterministic and source-safe, but the May 19 RefactoringMiner ablation showed it is not a quality backend.

### Existing Storage Surface

`crates/ctxhelm-index/src/storage.rs` already persists source-free semantic vector metadata through:

- `StorageSemanticVectorRecord`
- `StorageSemanticIndexReport`
- `persist_semantic_vector_records`
- `semantic_vectors` table

The storage table records provider, model, dimensions, distance metric, path, safe hash, privacy status, and vector JSON. It does not store source text.

### Existing Compiler And CLI Surface

The compiler already consumes semantic candidates in:

- `crates/ctxhelm-compiler/src/planning.rs`
- `crates/ctxhelm-compiler/src/ranking.rs`
- `crates/ctxhelm-compiler/src/policy.rs`
- `crates/ctxhelm-compiler/src/eval.rs`

The CLI already exposes semantic controls in `crates/ctxhelm/src/main.rs`:

- `--semantic` on search, prepare-task, get-pack, eval, and feature export paths
- `ctxhelm semantic status`
- `ctxhelm index --semantic`

This means Phase 56 should be additive: keep existing command shapes and extend provider/status fields rather than adding a new daily UX.

## Backend Research

### Recommended Local Backend

`fastembed` is the best first production local backend candidate for this Rust workspace:

- Rust crate integration path.
- `TextEmbedding` interface for embedding generation.
- `TextEmbedding::list_supported_models()`.
- `InitOptions::new(EmbeddingModel::AllMiniLML6V2)` style model selection.
- ONNX-backed execution through `ort`.
- bring-your-own model and reranker structs for later expansion.

Source: https://docs.rs/fastembed/

### Integration Decision

Use an optional Cargo feature for the real provider:

- feature name: `local-embeddings`
- provider id: `local_fastembed`
- default provider remains `local_hash`
- `local_hash` remains deterministic test/scaffold behavior

Why optional:

- Keeps default builds lightweight.
- Avoids requiring ONNX/runtime model downloads for normal test runs.
- Preserves local-first behavior without introducing remote model calls.
- Allows release gates to run deterministic hash-provider tests unless explicitly enabling the backend feature.

### Provider Contract Shape

Phase 56 should introduce an internal provider abstraction in `ctxhelm-index`:

- provider id
- model id
- dimensions
- distance metric
- privacy status
- health/status diagnostics
- document embedding
- query embedding

The abstraction can start private to `ctxhelm-index` and become public only when later phases need it.

## Product Proof Constraint

Phase 56 is not allowed to claim retrieval lift by itself. Success means:

- real local backend can be built and inspected
- source-free metadata is persisted
- `local_hash` is clearly labeled as scaffold/test behavior
- semantic candidates preserve provider provenance
- no cloud embeddings are used

Quality lift is measured in Phase 60 after semantic documents and fusion controls exist.

## Risks And Mitigations

| Risk | Mitigation |
|------|------------|
| `fastembed` adds heavy dependencies or model downloads | Put it behind `local-embeddings` feature and keep `local_hash` default |
| Tests become flaky due to network/model cache | Unit-test provider selection and metadata without downloading models; add feature-gated smoke only when local model cache is available |
| Provider status overclaims quality | Add explicit `qualityBackend` / scaffold labeling and update docs to say `local_hash` is not a quality backend |
| Source text leaks into storage or reports | Extend existing sentinel tests and storage tests to assert source text is absent |
| Semantic candidates crowd out anchors | Keep Phase 56 limited to backend/provenance; ranking changes remain bounded by existing semantic secondary-signal behavior |

## Implementation Targets

Likely files:

- `crates/ctxhelm-index/Cargo.toml`
- `crates/ctxhelm-index/src/semantic.rs`
- `crates/ctxhelm-index/src/storage.rs`
- `crates/ctxhelm-core/src/contracts.rs`
- `crates/ctxhelm-compiler/src/policy.rs`
- `crates/ctxhelm/src/main.rs`
- `docs/semantic.md`
- `docs/policy-embedding.md`
- `scripts/smoke-semantic.sh`
- `scripts/check-release-docs.sh`

## Validation Notes

Phase 56 validation should prove three layers:

1. **Contract tests**: provider metadata is explicit, `local_hash` is scaffold-labeled, and disabled-by-default semantics remain unchanged.
2. **Storage tests**: vector/provider metadata remains source-free and records provider/model/dimensions/freshness.
3. **CLI/report tests**: `ctxhelm semantic status` and `ctxhelm index --semantic` expose provider status and do not imply cloud usage or quality lift.

Feature-gated `local-embeddings` tests may be added, but the default workspace test suite must not require a model download.

## RESEARCH COMPLETE
