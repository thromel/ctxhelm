---
phase: 9
plan: 3
title: Persist reproducibility metadata and privacy diagnostics
status: complete
---

# Plan 3: Reproducibility Metadata

## Goal

Make benchmark reports reproducible and privacy-auditable without storing source text.

## Tasks

- Include suite ID, generated timestamp, repo IDs, revision ranges, limits, ranking budgets, mode, target agent, role filters, and evaluated commit counts.
- Include skipped/excluded changed-path counts per repository.
- Keep privacy status local-only.
- Avoid source snippets, prompt text, and commit subjects.

## Verification

- Compiler and CLI tests assert report shape and absence of source/prompt fields.
