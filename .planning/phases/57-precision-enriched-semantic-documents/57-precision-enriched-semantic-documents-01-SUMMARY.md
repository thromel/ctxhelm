---
phase: 57
title: Precision-Enriched Semantic Documents
status: complete
completed_at: 2026-05-20
commit_scope: pending
requirements_addressed:
  - PREC-01
  - PREC-02
  - PREC-03
  - PREC-04
---

# Phase 57 Summary: Precision-Enriched Semantic Documents

## What Changed

- Added shared source-free semantic document contracts in `ctxpack-core`.
- Added index-side semantic document construction from safe file metadata, symbols, dependency edges, related tests, docs, and precision overlays.
- Replaced semantic search/vector inputs with source-free semantic document text instead of raw file bodies.
- Added precision status reporting through semantic provider status.
- Added semantic result provenance fields: document ID, matched facets, and precision status.
- Updated ranking evidence to include semantic facet labels in source-free edge metadata.
- Updated docs and semantic smoke coverage for the new source-free behavior.

## Privacy Boundary

Semantic documents and persisted vector metadata now report `sourceTextLogged=false` and are built from bounded source-free facets. The implementation still reads safe local files where existing parsers require it, but raw source bodies are not exported, embedded, or persisted through the semantic document path.

## Verification

- `cargo test -p ctxpack-core --no-fail-fast`
- `cargo test -p ctxpack-index semantic --no-fail-fast`
- `cargo test -p ctxpack-compiler semantic --no-fail-fast`
- `CTXPACK_BIN=target/debug/ctxpack bash scripts/smoke-semantic.sh`
- `CTXPACK_BIN=target/debug/ctxpack bash scripts/smoke-precision.sh`
- `cargo test --workspace --no-fail-fast`
- `cargo run -p ctxpack -- --help`

All listed verification passed.

## Notes

- `local_hash` remains deterministic scaffold behavior, not a quality semantic backend.
- Precision overlays are additive. Missing overlays report `unavailable` and do not break semantic document generation.
- The semantic smoke now uses structural symbol/path terms because source bodies are no longer part of semantic input.
