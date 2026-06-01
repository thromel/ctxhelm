---
phase: 57
title: Precision-Enriched Semantic Documents
date: 2026-05-20
status: research
requirements:
  - PREC-01
  - PREC-02
  - PREC-03
  - PREC-04
---

# Phase 57 Research: Precision-Enriched Semantic Documents

## Objective

Turn the Phase 56 semantic backend from a file-level metadata retriever into a source-free semantic document layer that can carry safe structural evidence: symbols, roles, language, imports, exports, related tests, docs/cards, precision edges, and provider status.

The goal is not to add another search surface. The goal is to make semantic retrieval documents match the context compiler's actual evidence model while preserving the product boundary: local-first, read-only, source-safe by default, and no cloud behavior unless a later policy explicitly permits it.

## Current State

Phase 56 shipped:

- `SemanticProviderConfig`, `SemanticSearchReport`, `SemanticVectorRecord`, and provider status reporting.
- `local_fastembed` behind `local-embeddings`.
- `local_hash` renamed as a deterministic scaffold rather than a quality semantic backend.
- Source-free provider metadata and smoke coverage.

Existing related surfaces:

- `crates/ctxhelm-index/src/semantic.rs` builds safe vector records from file metadata.
- `crates/ctxhelm-index/src/dependencies.rs` imports `.ctxhelm/precision-edges.json` as safe overlay edges.
- `crates/ctxhelm-index/src/symbols.rs` extracts safe symbol names, signatures, kinds, paths, and line ranges.
- `crates/ctxhelm-compiler/src/planning.rs` currently consumes raw `SemanticSearchResult` values directly.
- `crates/ctxhelm-core/src/contracts.rs` has retrieval signal and evaluation contracts, but no typed semantic document contract.

## Problem

The current semantic record is too thin. It can represent that a file exists with role/language/provider metadata, but it does not provide enough structured evidence for a world-class context compiler:

- It cannot explain why semantic retrieval matched a task.
- It cannot distinguish a document that matched via a symbol, test, dependency, or precision edge.
- It cannot report precision backend freshness/availability independently from semantic provider availability.
- It cannot create source-free embedding text that is richer than path/role/language metadata.
- It gives the compiler too little evidence for fusion and evaluation.

## Research Findings

### Source-free semantic documents should be typed

The semantic layer should create stable document objects before vectorization or ranking. These should be safe to log, cache, and emit in diagnostics because they exclude source bodies. They can include:

- Path, role, language, package, and file hash.
- Exported symbols and signatures.
- Test names or test file relations.
- Import/export names and safe dependency edge labels.
- Documentation card titles and source links.
- Precision edge types and source backend status.

This document becomes the join point between lexical, graph, precision, semantic, and memory signals.

### Precision evidence must be additive and degradable

The system already supports precision edges through `.ctxhelm/precision-edges.json`. Phase 57 should not require SCIP/LSP to be installed. The correct behavior is:

- No precision file: semantic documents still build with `precision_status=unavailable`.
- Stale or invalid precision file: documents still build, diagnostics explain degradation.
- Valid precision file: precision facts enrich relevant documents and dependency signals.

### Semantic enrichment should not bypass privacy policy

Embedding text must remain source-safe by default. Safe text can contain names, signatures, dependency labels, test paths, and docs/card summaries already generated under ctxhelm policy. It must not embed full source bodies by default.

### Compiler integration should preserve selective retrieval

The compiler should not automatically inflate packs because richer semantic documents exist. Enrichment should improve candidate evidence and ranking explanations, not increase context size by default.

## Design Direction

Add a typed semantic document builder in the index crate and consume it from compiler planning:

- `SemanticDocument` contract in `ctxhelm-core`.
- `SemanticDocumentFacet` or equivalent typed evidence records.
- `PrecisionStatusReport` contract.
- Index-side builder that joins inventory, symbols, related tests, docs/cards, and dependency/precision edges.
- Compiler-side use in semantic ranking, explanations, and context pack evidence.

## Risks

- Source leakage: mitigated by tests that assert semantic documents do not include source file content.
- Context bloat: mitigated by using documents for evidence and ranking, not raw pack inclusion.
- Precision brittleness: mitigated by unavailable/stale/degraded status rather than hard failure.
- Duplicate graph evidence: mitigated by stable IDs and source signal labels.

## Acceptance Criteria

- Semantic documents exist as typed contracts and can be generated from a repo.
- Documents include safe structural facets from symbols, tests, docs/cards, and precision edges when available.
- Precision status is observable and non-fatal.
- `prepare_task` and related pack evidence can use enriched semantic evidence.
- Tests prove source-free behavior and degraded precision behavior.
