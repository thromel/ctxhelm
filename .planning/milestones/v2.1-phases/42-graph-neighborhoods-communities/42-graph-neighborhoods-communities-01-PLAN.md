# Phase 42 Plan: Graph Neighborhoods & Communities

**Created:** 2026-05-18
**Status:** In Progress

## Scope

Implement GRAPH-01, GRAPH-02, and GRAPH-04:
- graph neighborhood contracts
- source-free graph report generation
- budget caps and diagnostics
- CLI/docs/smoke coverage

## Steps

1. Add graph node/edge/community contracts.
2. Add compiler graph report builder using existing dependency/test/memory and
   feedback APIs.
3. Add `ctxpack graph neighborhood`.
4. Add docs and smoke coverage.
5. Run formatting, smokes, CLI help, and workspace tests.

## Verification

- `cargo fmt --all`
- `bash scripts/smoke-graph.sh`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxpack -- graph neighborhood --help`
- `cargo test --workspace`

## Non-goals

- No recursive graph expansion by default.
- No source text in graph artifacts.
- No policy-rank changes yet.
