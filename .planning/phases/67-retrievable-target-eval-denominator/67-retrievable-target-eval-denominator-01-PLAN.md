# Phase 67 Plan: Retrievable Target Eval Denominator

Date: 2026-05-30

## Goal

Make historical retrieval metrics measure files that could actually be retrieved from the parent snapshot while preserving a source-free audit trail of all safe changed files.

## Problem

Phase 66 fixed Test Recall@10, but ctxpack still showed many `no_candidate_signal` gaps for newly added planning, docs, and script files. Those files were present in the final patch but absent from the parent snapshot used for retrieval. Counting them in Recall@K mixed two different capabilities:

- selecting existing repository context before editing;
- inventing paths for new files that do not exist yet.

ctxpack is a context broker, so the release proof should evaluate retrieval over existing context and report newly-added files separately.

## Plan

1. Keep `safeChangedFiles` as the full source-free patch audit list.
2. Add `retrievalTargetFiles` as the subset of safe changed files that exist in the parent evaluation snapshot.
3. Use `retrievalTargetFiles` for file recall, lexical recall, MRR, token ROI, signal-only metrics, role recall, missing files, and retrieval-gap summaries.
4. Add a regression test proving added docs are not counted as retrievable misses when an existing source file is modified in the same commit.
5. Re-run the two-repo proof and keep the release gate blocked unless every corpus beats lexical.

## Success Criteria

- Historical eval JSON includes source-free `retrievalTargetFiles`.
- Added files remain visible in `safeChangedFiles`.
- New-file-only artifacts do not create false retrieval misses.
- Product proof remains honest about remaining lexical gaps.
