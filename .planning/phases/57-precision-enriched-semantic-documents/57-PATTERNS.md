---
phase: 57
title: Precision-Enriched Semantic Documents
date: 2026-05-20
status: patterns
---

# Phase 57 Patterns

## Existing Patterns To Reuse

- Contracts live in `crates/ctxpack-core/src/contracts.rs` and use serde-friendly camelCase fields.
- Index reports are built in `crates/ctxpack-index/src/*.rs` and return typed report structs rather than string-only output.
- Compiler planning reads index reports and converts them into ranked evidence in `crates/ctxpack-compiler/src/planning.rs`.
- Privacy-sensitive outputs expose hashes, paths, roles, language, status, and reasons, not source bodies.
- Smoke tests live in `scripts/smoke-*.sh` and verify CLI behavior against local fixtures.

## Closest Code Analogs

- `crates/ctxpack-index/src/semantic.rs`: provider config, vector records, search reports, source-free cache metadata.
- `crates/ctxpack-index/src/dependencies.rs`: precision overlay loading, safe path validation, degraded import behavior.
- `crates/ctxpack-index/src/symbols.rs`: safe symbol extraction and report shape.
- `crates/ctxpack-compiler/src/planning.rs`: central fusion point for semantic, lexical, graph, test, history, and memory signals.
- `crates/ctxpack-core/src/contracts.rs`: shared stable API surface for CLI/MCP/eval.

## Implementation Notes

- Keep semantic document generation deterministic and source-free.
- Prefer additive fields over replacing existing semantic contracts.
- Do not require local embeddings or precision backends for document generation.
- Treat precision as a provider status and evidence source, not as a mandatory index dependency.
- Add tests near the crates that own behavior: contract serialization in core, document construction in index, ranking evidence in compiler.
