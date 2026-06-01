---
phase: 64
title: Gap-Family Retrieval Improvements
date: 2026-05-30
status: research
requirements:
  - GAP-01
  - GAP-02
  - GAP-03
  - GAP-04
---

# Phase 64 Research: Gap-Family Retrieval Improvements

## Objective

Convert measured retrieval gap families into targeted retrieval fixes with
before/after proof on the fixed v2.5 two-repo corpus.

## Current Evidence

Phase 63 proved that a broad local metadata reranker is not safe to promote yet.
It improved RefactoringMiner aggregate Recall@10 but regressed ctxhelm and
demoted protected evidence.

The most actionable Phase 63 gaps are:

- RefactoringMiner: `no_candidate_signal` for
  `src/main/java/org/refactoringminer/astDiff/matchers/wrappers/*.java`.
- RefactoringMiner: `no_candidate_signal` for
  `src/main/java/gr/uom/java/xmi/decomposition/*.java`.
- ctxhelm: docs/planning `no_candidate_signal`.
- ctxhelm: compiler `ranked_below_budget_dependency`.

## Problem

Broad reranking changes can hide specific retrieval failures. Phase 64 should
instead identify why high-impact gold files never become candidates, then add a
small retriever, query facet, or graph/test/history edge that makes those files
eligible without weakening anchors or exact evidence.

## Design Direction

Start with the RefactoringMiner Java wrapper gap because it is:

- repeated across commits;
- source-file oriented;
- policy-safe and current-reachable;
- likely tied to package/path/symbol terminology that should be discoverable.

The likely fix surface is one of:

- query construction terms that preserve path/package tokens from commit titles;
- Java package/class lexical expansion;
- graph seed expansion around discovered Java symbols;
- source-free path-family candidate backfill under tight budgets.

Any fix must be measured against:

- RefactoringMiner Recall@10 and gap count for the wrapper family;
- ctxhelm Recall@10 to catch collateral regressions;
- test recall separately from source recall;
- protected evidence miss rate.

## Acceptance Criteria

- Gap families are grouped into named source-free work items.
- At least one high-impact RefactoringMiner gap family improves in before/after
  reports.
- Test recall remains visible as its own metric.
- Graph expansion remains budgeted and exact-seed-safe.
