---
phase: 58
title: Query Construction And Hybrid Fusion Controls
date: 2026-05-20
status: research
requirements:
  - QUERY-01
  - QUERY-02
  - QUERY-03
  - QUERY-04
---

# Phase 58 Research: Query Construction And Hybrid Fusion Controls

## Objective

Make query construction explicit, inspectable, and reusable across lexical, semantic, symbol, graph, test, history, and memory retrieval. Then add hybrid fusion controls that keep anchors and exact evidence dominant when they should dominate.

## Current State

The planner already fuses multiple signals:

- Anchors and current diff.
- Lexical results.
- Semantic results when enabled.
- Symbol results.
- Related tests.
- Co-change hints and dependency edges.
- Memory cards.

The weakness is that query construction is implicit. Each retriever derives its own strings or receives partially processed task text. That makes regressions hard to debug and makes benchmark variants less comparable.

## Problem

Recent eval work showed that enabling a semantic path does not automatically improve recall. One likely bottleneck is that the semantic query may not be constructed from the same high-value facets as lexical/graph/test retrieval:

- Explicit paths and symbols should be anchored, not buried in task prose.
- Stack traces and error messages should be parsed into exact terms and path hints.
- Commit subjects should preserve identifiers and quoted terms.
- Current diff paths should contribute hard anchors.
- Domain phrases should remain available for semantic search.

Without a query construction trace, it is difficult to know whether a failure is caused by bad retrieval, bad fusion, bad semantic provider quality, bad query text, or an eval case that does not exercise semantic behavior.

## Design Direction

Introduce a typed query construction stage:

- `QueryFacet`: path, symbol, exact term, error text, route/config key, domain phrase, doc phrase, commit clue, current diff anchor.
- `QueryConstructionTrace`: original task plus the facets sent to each retriever.
- `RetrieverQuerySet`: lexical terms, semantic phrases, symbol names, graph seeds, history terms, test terms.

Then update the retrieval planner to consume the same structured query set for every retriever. Fusion should record which query facets created each candidate signal.

## Fusion Controls

Hybrid retrieval needs guardrails:

- Explicit user paths and current diff anchors should not be displaced by weak semantic hits.
- Semantic-only candidates should be capped unless there is no lexical/symbol/graph evidence.
- Precision/dependency expansion should be budgeted and local to high-confidence seeds.
- Every candidate should carry source signal labels for eval and debugging.

## Evaluation Direction

Phase 58 should add fixed-budget paired variants so we can compare:

- lexical only
- lexical plus graph
- lexical plus semantic
- lexical plus precision-enriched semantic
- full hybrid

These variants must use the same query construction trace so any differences are attributable to retrievers/fusion, not accidental prompt/query drift.

## Risks

- Too much explainability surface can bloat CLI/MCP outputs; traces should be opt-in or summarized by default.
- Query facets can overfit English task phrasing; tests should include stack traces, paths, commit-like prompts, and conceptual prompts.
- Fusion controls can suppress legitimate semantic discoveries; caps should be visible and configurable in eval.

## Acceptance Criteria

- Query construction is a typed stage with source-free traces.
- Planner retrievers consume the shared query set.
- Candidate evidence can point back to query facets.
- Hybrid controls prevent semantic drift from suppressing anchors.
- Eval variants can compare retrieval paths under the same fixed query trace.
