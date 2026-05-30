---
phase: 63
title: Reranker And Fusion Promotion
date: 2026-05-30
status: research
requirements:
  - RANK-01
  - RANK-02
  - RANK-03
  - RANK-04
---

# Phase 63 Research: Reranker And Fusion Promotion

## Objective

Make reranker and fusion variants measurable, source-safe, and promotion-gated so ctxpack can improve retrieval quality without weakening explicit evidence protections.

## Current State

v2.4 added provider and reranker policy gates. Phase 61 established a two-repo retrieval baseline. Phase 62 showed that production local embeddings are available and source-safe, but did not improve Recall@10 on the fixed corpus enough to promote them as a default.

Current measured state:

- RefactoringMiner default Recall@10 is near lexical parity.
- ctxpack default still trails lexical on its own repo.
- `local_hash` is deterministic scaffold behavior.
- `local_fastembed` is a real local provider, but remains opt-in because runtime increased without recall lift.

## Problem

The product needs quality lift, not more retrieval surface area. Reranker and fusion experiments can improve ordering, but they are risky if they:

- crowd out user anchors, current diff files, exact lexical matches, or high-confidence symbols;
- make reports source-bearing;
- optimize a single metric while regressing critical cases;
- promote slower defaults without token ROI or runtime proof.

## Design Direction

Use the existing `ctxpack eval benchmark` surface and add promotion-grade comparison rather than a second evaluator.

Track every variant with source-free metadata:

- Recall@K and MRR@K.
- precision proxy and token ROI.
- test recall where labels exist.
- runtime and per-repo deltas.
- anchor/exact/symbol protection violations.
- named wins and regressions.
- policy privacy status.

Promotion should be blocked if a variant improves aggregate recall but violates protected evidence invariants or regresses named critical cases.

## Acceptance Criteria

- Maintainers can compare reranker/fusion variants without adding MCP tools.
- Reports show whether a variant beats, matches, or trails baseline and lexical.
- Protected anchors, current diff, exact lexical matches, and strong symbols cannot be demoted by weaker signals.
- Named regressions are visible and can block promotion.
