# Phase 43 Plan: Graph-Aware Policy & Embedding Controls

**Created:** 2026-05-18
**Status:** In Progress

## Scope

Implement GRAPH-03 and EMBED-01 through EMBED-04:
- semantic provider status
- graph/semantic policy experiment comparison
- explicit cloud-disabled policy labels
- docs/smoke coverage

## Steps

1. Add semantic provider and policy experiment contracts.
2. Add compiler report builders.
3. Add `ctxpack semantic status` and `ctxpack eval policy experiments`.
4. Add docs and smoke coverage.
5. Run formatting, smokes, CLI help, and workspace tests.

## Verification

- `cargo fmt --all`
- `bash scripts/smoke-policy-embedding.sh`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxpack -- semantic status --help`
- `cargo run -p ctxpack -- eval policy experiments --help`
- `cargo test --workspace`

## Non-goals

- No default ranking changes.
- No cloud embedding/reranking enablement.
- No hosted policy service.
