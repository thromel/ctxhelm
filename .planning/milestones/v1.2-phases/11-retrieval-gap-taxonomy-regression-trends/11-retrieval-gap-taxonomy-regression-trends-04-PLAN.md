---
phase: 11
plan: 4
title: Validate trend reports on repeated real-repo benchmark runs
status: complete
---

# Plan 4: Validation

## Goal

Prove the comparison path works through the CLI and remains source-free.

## Tasks

- Run targeted compiler gap taxonomy tests.
- Run CLI comparison tests.
- Run full workspace tests before commit.
- Update docs for compare command and gap interpretation.

## Verification

- `cargo test -p ctxhelm-compiler ablation_historical_eval_groups_source_free_retrieval_gaps -- --nocapture`
- `cargo test -p ctxhelm --test cli_compat eval_compare_reports_source_free_metric_and_gap_deltas -- --nocapture`
- `cargo test --workspace`
