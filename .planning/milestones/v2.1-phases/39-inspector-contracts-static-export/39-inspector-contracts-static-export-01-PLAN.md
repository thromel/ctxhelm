# Phase 39 Plan: Inspector Contracts & Static Export

**Created:** 2026-05-18
**Status:** In Progress

## Scope

Implement INSP-01 through INSP-04:
- typed source-free inspector contracts
- compiler conversion from plan plus pack to inspector view
- CLI static export for JSON, Markdown, and HTML
- tests and docs proving source-bearing snippets are separated

## Steps

1. Add inspector data contracts to `ctxpack-core`.
2. Add compiler helpers to build and render inspector views.
3. Add `ctxpack inspector export` CLI command.
4. Add focused tests for serialization and source sentinel separation.
5. Add docs and update release documentation checks.
6. Run formatting, focused tests, CLI help, and workspace tests as feasible.

## Verification

- `cargo fmt --all`
- Focused compiler/core tests for inspector behavior
- `cargo run -p ctxpack -- inspector export --help`
- `cargo test --workspace`

## Non-goals

- No persistent UI server.
- No autonomous editing.
- No cloud embedding/reranking behavior.
- No broad rewrite of existing pack contracts.
