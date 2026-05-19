# Phase 56 Pattern Map: Production Local Semantic Backend

**Phase:** 56 - Production Local Semantic Backend
**Status:** Complete

## Existing Patterns To Reuse

### Semantic provider and vectorization

Closest file: `crates/ctxpack-index/src/semantic.rs`

Reuse:

- `SemanticProviderConfig` for public provider metadata.
- `SemanticOptions` for explicit enablement and limit handling.
- `SemanticSearchReport` diagnostics/cache/privacy shape.
- `SemanticVectorRecord` as the bridge into storage.
- `normalized_provider` pattern for backward-compatible defaults.

Do not reuse as quality backend:

- `vectorize_text` hash-vector logic should remain the deterministic `local_hash` scaffold.

### Source-safe reads

Closest files:

- `crates/ctxpack-index/src/policy.rs`
- `crates/ctxpack-index/src/inventory.rs`

Reuse:

- `read_safe_source`
- `SourceReadStatus`
- `SOURCE_READ_MAX_BYTES`
- safe inventory filters for generated, sensitive, and ignored paths

Every local embedding path must go through these existing source-read gates.

### Storage persistence

Closest file: `crates/ctxpack-index/src/storage.rs`

Reuse:

- `StorageSemanticVectorRecord`
- `persist_semantic_vector_records`
- semantic migration/table pattern
- existing source-free storage tests

Phase 56 should extend metadata only if needed; avoid storing raw semantic document text in SQLite.

### Compiler policy/status reports

Closest file: `crates/ctxpack-compiler/src/policy.rs`

Reuse:

- `semantic_provider_status_report`
- `SemanticProviderStatusReport`
- `SemanticUsageSummary`
- existing policy experiment rows

Provider status should be extended here rather than creating a second report path.

### CLI rendering

Closest file: `crates/ctxpack/src/main.rs`

Reuse:

- `SemanticArgs`
- `SemanticCommand::Status`
- `render_semantic_provider_status`
- `print_semantic_storage_report`

Keep existing command names and flags.

### Documentation and release checks

Closest files:

- `docs/semantic.md`
- `docs/policy-embedding.md`
- `docs/storage.md`
- `scripts/smoke-semantic.sh`
- `scripts/check-release-docs.sh`

Reuse:

- explicit `--semantic` language
- cloud-disabled proof language
- source-free storage proof
- smoke script pattern

## Files Most Likely To Change

| File | Role |
|------|------|
| `crates/ctxpack-index/Cargo.toml` | optional provider dependency/feature |
| `crates/ctxpack-index/src/semantic.rs` | provider abstraction, local hash labeling, local embedding backend |
| `crates/ctxpack-index/src/storage.rs` | source-free provider/vector metadata if schema needs extension |
| `crates/ctxpack-core/src/contracts.rs` | provider status/report fields |
| `crates/ctxpack-compiler/src/policy.rs` | semantic status report construction |
| `crates/ctxpack/src/main.rs` | CLI output/status rendering |
| `docs/semantic.md` | user-facing provider docs |
| `docs/policy-embedding.md` | policy/status docs |
| `scripts/smoke-semantic.sh` | deterministic smoke coverage |
| `scripts/check-release-docs.sh` | release-doc guard strings |

## Existing Tests To Extend

- `semantic_search_is_disabled_by_default`
- `semantic_search_finds_conceptual_safe_files`
- `persists_semantic_vectors_without_source_text`
- ranking tests that keep semantic as secondary signal
- CLI compatibility tests that inspect semantic docs/scripts

