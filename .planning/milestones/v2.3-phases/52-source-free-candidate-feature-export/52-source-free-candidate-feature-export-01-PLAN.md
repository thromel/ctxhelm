---
phase: 52
title: Source-Free Candidate Feature Export
status: complete
---

# Plan

## Goal

Maintainers can export privacy-safe candidate feature rows for learning, diagnostics, and paired analysis.

## Tasks

- Add source-free candidate feature export and row contracts.
- Generate feature rows from existing context-plan retrieval candidates.
- Persist exports under local ctxpack state by repo ID and export ID.
- Add CLI commands for export, list, inspect, compare, and delete.
- Add Markdown/JSON renderers and lifecycle tests.
- Document the feature export surface and privacy boundary.

## Verification

- `cargo fmt --all`
- `CARGO_INCREMENTAL=0 cargo check --workspace`
- `CARGO_INCREMENTAL=0 cargo test -p ctxpack-compiler candidate_feature_export_persists_source_free_rows -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo test -p ctxpack eval_features_exports_and_manages_source_free_rows -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo run -p ctxpack -- eval features --help`
- `CARGO_INCREMENTAL=0 cargo run -p ctxpack -- --help`
- `CARGO_INCREMENTAL=0 cargo test --workspace`
- `git diff --check`
