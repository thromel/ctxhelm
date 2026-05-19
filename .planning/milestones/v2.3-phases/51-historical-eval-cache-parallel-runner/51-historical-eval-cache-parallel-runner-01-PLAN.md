---
phase: 51
title: Historical Eval Cache & Parallel Runner
status: complete
---

# Plan

## Goal

Maintainers can run large-history evals faster through cache reuse, parallel samples, and runtime diagnostics.

## Tasks

- Add historical eval options for source-free report cache reuse, force refresh, and parallelism.
- Implement cache lookup/write keyed by repo, eval range options, refs, and cache schema version.
- Split per-commit historical eval work into deterministic sequential/parallel runner paths.
- Add runtime diagnostics to historical eval reports, benchmark reports, and CLI Markdown output.
- Extend benchmark manifest defaults/repo overrides with `cacheEnabled`, `forceRefresh`, and `parallelism`.
- Add focused tests for cache reuse, force refresh, deterministic metadata, and source-free cache contents.

## Verification

- `cargo fmt --all`
- `CARGO_INCREMENTAL=0 cargo check --workspace`
- `CARGO_INCREMENTAL=0 cargo test -p ctxpack-compiler historical_eval_reuses_source_free_cache_and_parallelism_metadata -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo test -p ctxpack-compiler benchmark_suite_runs_multiple_repos_with_source_free_metadata -- --nocapture`
- `cargo run -p ctxpack -- eval history --help`
- `cargo run -p ctxpack -- eval health --help`
- `git diff --check`
