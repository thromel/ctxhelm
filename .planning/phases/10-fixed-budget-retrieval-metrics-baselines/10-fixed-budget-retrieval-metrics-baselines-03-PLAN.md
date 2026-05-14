---
phase: 10
plan: 3
title: Add signal ablation and token ROI reporting
status: complete
---

# Plan 3: Signal Ablation And Token ROI

## Goal

Quantify which context budgets are worth their estimated token cost and keep signal ablation output visible in reports.

## Tasks

- Preserve signal ablation output in JSON and Markdown.
- Add `TokenRoiMetric` for brief, standard, and deep budgets.
- Report useful targets, safe targets, recall, estimated tokens, and useful targets per 1k tokens.
- Mark larger packs that add no useful target labels over the previous budget.

## Verification

- Compiler and CLI tests assert `tokenRoi` rows are source-free and include budget efficiency fields.
