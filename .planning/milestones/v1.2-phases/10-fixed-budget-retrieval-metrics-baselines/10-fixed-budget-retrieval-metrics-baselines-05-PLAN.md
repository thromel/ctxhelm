---
phase: 10
plan: 5
title: Validate metrics against RefactoringMiner and the second real repo
status: complete
---

# Plan 5: Metric Validation

## Goal

Validate Phase 10 metrics with focused unit and CLI tests, then keep real-repo validation available through the Phase 9 suite runner.

## Tasks

- Run focused compiler tests for historical eval metrics.
- Run CLI compatibility tests for history JSON and benchmark Markdown.
- Run all compiler tests.
- Document the metric interpretation for RefactoringMiner-style benchmark suites.

## Verification

- `cargo test -p ctxhelm-compiler -- --nocapture`
- `cargo test -p ctxhelm --test cli_compat search_related_tests_dependencies_and_eval_history_emit_json_shapes -- --nocapture`
- `cargo test -p ctxhelm --test cli_compat eval_benchmark_runs_named_suite_source_free -- --nocapture`
