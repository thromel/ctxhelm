---
phase: 56
plan: 56-production-local-semantic-backend-01
title: Production Local Semantic Backend
subsystem: semantic-retrieval
status: completed
completed_at: 2026-05-19
tags:
  - semantic
  - local-embeddings
  - source-free-storage
  - provider-status
requirements_addressed:
  - SEM-01
  - SEM-02
  - SEM-03
  - SEM-04
key_files:
  - Cargo.toml
  - Cargo.lock
  - crates/ctxpack-index/src/semantic.rs
  - crates/ctxpack-index/src/storage.rs
  - crates/ctxpack-core/src/contracts.rs
  - crates/ctxpack-compiler/src/policy.rs
  - crates/ctxpack/src/main.rs
  - docs/semantic.md
  - docs/policy-embedding.md
  - docs/storage.md
  - docs/release.md
  - scripts/smoke-semantic.sh
  - scripts/smoke-policy-embedding.sh
  - scripts/check-release-docs.sh
metrics:
  workspace_tests: pass
  local_embeddings_feature_check: pass
  semantic_smoke: pass
  release_docs_check: pass
---

# Summary: Production Local Semantic Backend

## Commits

| Commit | Purpose |
|--------|---------|
| `7e1386a` | Adds semantic provider role/status metadata, feature-gated `local_fastembed`, source-free vector metadata coverage, CLI status rendering, and docs/smoke updates. |

## What Changed

- Extended `SemanticProviderConfig` with additive camelCase fields: `providerRole`, `qualityBackend`, `localOnly`, and `available`.
- Kept `local_hash` as the default provider and labeled it as `deterministic_scaffold` with `qualityBackend: false`.
- Added optional `local_fastembed` provider metadata and real fastembed execution path behind the `ctxpack-index/local-embeddings` Cargo feature.
- Added unavailable-provider diagnostics for feature-disabled or unsupported providers without falling back silently or using cloud services.
- Extended semantic provider status output with provider availability, degraded state, cache location, local-only status, and scaffold/quality status.
- Preserved source-free semantic vector persistence while covering the new provider metadata shape in storage tests.
- Updated semantic, policy/embedding, storage, and release docs to state the Phase 56 boundary clearly.
- Updated smoke/release checks to assert scaffold labeling, cloud-disabled status, and source-free semantic metadata.

## Verification

- `cargo fmt --all`
- `cargo check -p ctxpack-index`
- `cargo check -p ctxpack-index --features local-embeddings`
- `cargo test -p ctxpack-index semantic_search_is_disabled_by_default -- --nocapture`
- `cargo test -p ctxpack-index semantic_search_finds_conceptual_safe_files -- --nocapture`
- `cargo test -p ctxpack-index semantic_provider_unavailable_without_feature -- --nocapture`
- `cargo test -p ctxpack-index persists_semantic_vectors_without_source_text -- --nocapture`
- `cargo test -p ctxpack-compiler semantic_provider_status -- --nocapture`
- `cargo run -p ctxpack -- semantic status --repo . --query "semantic backend provider" --format json`
- `cargo run -p ctxpack -- search "semantic backend provider" --repo . --semantic`
- `cargo run -p ctxpack -- prepare-task "improve semantic backend provider status" --repo . --semantic --no-trace`
- `cargo run -p ctxpack -- index --repo . --store-path <temp-store> --semantic`
- `cargo run -p ctxpack -- storage status --repo . --path <temp-store> --format json`
- `bash scripts/check-release-docs.sh`
- `bash scripts/smoke-policy-embedding.sh`
- `CTXPACK_BIN=target/debug/ctxpack bash scripts/smoke-semantic.sh`
- `cargo run -p ctxpack -- --help`
- `cargo test --workspace`

## Deviations From Plan

- `fastembed 5.13.4` was not used because its `ort` dependency requires Rust 1.88 while this workspace declares Rust 1.87. The implementation uses `fastembed =4.9.0`, which compiles under the current workspace toolchain and still provides `JinaEmbeddingsV2BaseCode`.
- `ctxpack search` and `ctxpack prepare-task` do not currently expose a CLI flag to select `local_fastembed`; the provider path is present in the typed semantic API and feature-gated implementation. Provider policy and user-selectable backend routing remain appropriate for Phase 59.

## Self-Check

PASSED. Phase 56 satisfies SEM-01 through SEM-04 without enabling semantic defaults, cloud embeddings, cloud reranking, or source-bearing semantic storage.
