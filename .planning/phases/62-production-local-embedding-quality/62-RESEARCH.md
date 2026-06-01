---
phase: 62
title: Production Local Embedding Quality
date: 2026-05-22
status: research
requirements:
  - EMBED-01
  - EMBED-02
  - EMBED-03
  - EMBED-04
---

# Phase 62 Research: Production Local Embedding Quality

## Objective

Evaluate whether a real local embedding backend improves retrieval quality over default and lexical baselines on the Phase 61 two-repo corpus. This phase must keep cloud transfer disabled and must not promote semantic defaults unless measured evidence supports it.

## Current State

v2.4 added:

- `local_hash` deterministic scaffold provider.
- Feature-gated `local_fastembed` provider.
- Source-free semantic vector metadata.
- Semantic status and provider policy reports.
- MCP semantic provider/model/dimension controls.

Phase 61 baseline:

- RefactoringMiner default Recall@10 `0.7767`, lexical `0.7792`.
- ctxhelm default Recall@10 `0.2270`, lexical `0.2742`.

## Problem

`local_hash` is useful for deterministic testing but is not a quality backend. A production local embedding backend may improve conceptual retrieval, but it can also increase runtime or crowd out exact evidence. Phase 62 must measure that tradeoff directly.

## Design Direction

Use the same benchmark corpus and compare:

- default no-semantic
- lexical baseline
- `local_hash`
- `local_fastembed`
- `local_fastembed` with precision-enriched documents if available

Track:

- Recall@10 and MRR@10
- source/test recall
- runtime and cache behavior
- semantic signal saturation
- named wins/regressions
- cache path and inventory exclusion
- provider privacy status

## Acceptance Criteria

- Production local embedding backend can be evaluated from CLI with the `local-embeddings` feature.
- Cache and provider metadata are visible and source-safe.
- The report states beat/match/trail versus lexical and default.
- `local_hash` remains scaffold-labeled in docs and output.
