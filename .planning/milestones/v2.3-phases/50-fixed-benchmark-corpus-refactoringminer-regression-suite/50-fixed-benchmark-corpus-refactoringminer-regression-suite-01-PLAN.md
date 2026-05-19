---
phase: 50
title: Fixed Benchmark Corpus & RefactoringMiner Regression Suite
status: complete
---

# Plan

## Goal

Maintainers can define fixed source-free benchmark corpora and lock the first large-history regression suite.

## Tasks

- Extend benchmark suite contracts with source-free manifest identity fields.
- Add optional repo-level baseline metadata and comparison deltas.
- Add a locked RefactoringMiner v2.3 benchmark manifest.
- Update benchmark docs with v2.3 corpus semantics.
- Add focused tests to preserve backward-compatible source-free output.

## Verification

- `cargo check --workspace`
- `cargo test -p ctxpack-compiler benchmark_suite_runs_multiple_repos_source_free -- --nocapture`
- RefactoringMiner external run is intentionally deferred to the v2.3 optional large-repo gate because the locked suite is a multi-minute benchmark.
