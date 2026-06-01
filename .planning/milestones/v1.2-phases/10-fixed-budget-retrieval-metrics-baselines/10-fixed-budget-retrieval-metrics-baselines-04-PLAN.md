---
phase: 10
plan: 4
title: Expose benchmark reports through CLI JSON and Markdown outputs
status: complete
---

# Plan 4: CLI Report Surfaces

## Goal

Expose the new baseline and token ROI fields through both machine-readable JSON and human-readable Markdown.

## Tasks

- Render no-context recall and lift in `ctxhelm eval history` Markdown.
- Render token ROI rows in historical eval Markdown.
- Render no-context and token ROI summaries in benchmark suite Markdown.
- Keep JSON output sourced directly from typed report contracts.

## Verification

- CLI compatibility tests assert new JSON keys and Markdown phrases.
