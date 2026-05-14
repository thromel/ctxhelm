---
phase: 10
plan: 2
title: Add fixed-budget recall, useful-target, and missing-label metrics
status: complete
---

# Plan 2: Fixed-Budget Metrics

## Goal

Keep benchmark reports focused on fixed-budget target retrieval quality rather than broad repository search output.

## Tasks

- Reuse existing file Recall@5/10 and role Recall@5/10 metrics.
- Preserve useful-target precision-like metrics through `precisionAtK`.
- Preserve top missing file summaries and grouped retrieval failures.
- Ensure no-context and lexical baselines use the same K budget as ctxpack.

## Verification

- Historical eval tests assert fixed-budget recall, precision, MRR, baseline fields, and missing-label summaries remain present.
