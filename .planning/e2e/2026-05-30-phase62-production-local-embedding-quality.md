# Phase 62 E2E: Production Local Embedding Quality

Date: 2026-05-30

## Objective

Measure production local embedding retrieval on the Phase 61 two-repo corpus and
decide whether it should remain opt-in, be tuned, or be promoted.

## Commands

```bash
cargo test -p ctxpack-index semantic --features local-embeddings -- --nocapture
cargo test -p ctxpack-index policy_classifies_common_credentials_and_generated_families -- --nocapture
cargo test -p ctxpack-compiler benchmark_suite_runs_multiple_repos_with_source_free_metadata -- --nocapture

cargo run -p ctxpack --features local-embeddings -- \
  semantic status --repo . --semantic-provider local_fastembed --format json

cargo run -p ctxpack --features local-embeddings -- \
  semantic status --repo . --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q --format json

cargo run -p ctxpack --features local-embeddings -- \
  eval benchmark --config .ctxpack/e2e/phase62-default-config.json --format json

cargo run -p ctxpack --features local-embeddings -- \
  eval benchmark --config .ctxpack/e2e/phase62-local-hash-config.json --format json

cargo run -p ctxpack --features local-embeddings -- \
  eval benchmark --config .ctxpack/e2e/phase62-local-fastembed-config.json --format json
```

## Provider Status

`local_fastembed` default model:

- provider: `local_fastembed`
- model: `JinaEmbeddingsV2BaseCode`
- dimensions: `768`
- provider role: `production_local`
- quality backend: `true`
- provider status: `available`
- source text logged: `false`

`local_fastembed` eval model:

- provider: `local_fastembed`
- model: `AllMiniLML6V2Q`
- dimensions: `384`
- provider role: `production_local`
- quality backend: `true`
- provider status: `available`
- source text logged: `false`

Cache status:

- model cache defaults to repo `.ctxpack/cache/fastembed` inside a git repo, otherwise `CTXPACK_HOME/cache/fastembed`
- override supported through `CTXPACK_FASTEMBED_CACHE_DIR`
- query-time source-free vector cache is bounded in process
- expensive model candidate prefilter defaults to `128`
- override supported through `CTXPACK_FASTEMBED_DOCUMENT_LIMIT`

## Benchmark Results

| Variant | Provider role | Quality backend | RefactoringMiner Recall@10 | ctxpack Recall@10 | Total repo runtime |
| --- | --- | --- | ---: | ---: | ---: |
| Default, semantic off | `deterministic_scaffold` | false | 0.7767 | 0.2299 | 26.3s |
| `local_hash` | `deterministic_scaffold` | false | 0.7767 | 0.2299 | 57.8s |
| `local_fastembed` `AllMiniLML6V2Q` | `production_local` | true | 0.7767 | 0.2299 | 183.7s |

Per-repo runtime:

| Variant | RefactoringMiner runtime | ctxpack runtime |
| --- | ---: | ---: |
| Default, semantic off | 18.9s | 7.4s |
| `local_hash` | 34.9s | 22.9s |
| `local_fastembed` `AllMiniLML6V2Q` | 110.7s | 73.0s |

## Decision

Hold. `local_fastembed` is source-free and production-local, but it does not
improve Recall@10 on this corpus and runtime is much higher than default
retrieval.

Do not promote semantic defaults from Phase 62. Continue to Phase 63 reranker and
fusion promotion work, then Phase 64 gap-family retrieval fixes.
