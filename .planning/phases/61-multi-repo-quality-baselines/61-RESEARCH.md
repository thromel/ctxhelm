---
phase: 61
title: Multi-Repo Quality Baselines
date: 2026-05-22
status: research
requirements:
  - BASE-01
  - BASE-02
  - BASE-03
  - BASE-04
---

# Phase 61 Research: Multi-Repo Quality Baselines

## Objective

Make retrieval quality claims reproducible across more than one real repository. RefactoringMiner is a strong benchmark because it has rich history and known ctxpack misses, but a single repo can overfit policy decisions. Phase 61 should create a reusable multi-repo baseline artifact that keeps reports source-free and comparable.

## Current State

ctxpack can already run:

- `ctxpack eval history`
- `ctxpack eval baselines`
- `ctxpack eval benchmark`
- paired default/lexical/no-context/signal-only/ablation reports
- source-free gap summaries and token ROI

Recent RefactoringMiner proof:

- default Recall@10: `0.6355`
- `local_hash` Recall@10: `0.6355`
- lexical baseline Recall@10: `0.6665`
- semantic-only Recall@10: `0.3143`

This means semantic integration is no longer regressive, but production quality is not yet better than lexical.

## Design Direction

Use a manifest-driven multi-repo report. Each repo entry should include:

- repo label
- local path
- optional base/head revision
- commit limit
- ranking budget
- enabled variants
- expected privacy mode
- optional baseline thresholds

The report should aggregate:

- per-repo Recall@K, MRR@K, precision proxy, test recall, validation coverage
- runtime and cache stats
- lexical delta
- named wins, misses, regressions
- repeated gap families
- provider/reranker/semantic status
- privacy status

## First Repos

1. `../RefactoringMiner`
   - Known rich history.
   - Known lexical baseline still ahead.
   - Known MCP/source/test gap families.

2. Current ctxpack repo or another local real repo with enough history.
   - Use whichever has stable local availability and enough commits.
   - Avoid toy-only fixtures.

## Risks

- Multi-repo eval can become slow; keep `limit` and `parallelism` configurable.
- Repos may have dirty state; eval should use git history and parent snapshots, not working tree assumptions.
- A second repo may have weaker labels if commits are documentation-heavy; report low-information tasks explicitly.

## Acceptance Criteria

- A manifest-driven baseline command or report path exists.
- At least two real repos can be evaluated with one command/config.
- Reports are source-free and include per-repo plus aggregate quality.
- Output identifies whether ctxpack beats, matches, or trails lexical per repo.
