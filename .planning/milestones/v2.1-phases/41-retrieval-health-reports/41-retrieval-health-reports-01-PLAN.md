# Phase 41 Plan: Retrieval Health Reports

**Created:** 2026-05-18
**Status:** In Progress

## Scope

Implement HEALTH-01 through HEALTH-04:
- source-free retrieval health contract
- report generation from historical eval and feedback policy data
- CLI JSON/Markdown output
- docs and smoke coverage for no source leakage

## Steps

1. Add `RetrievalHealthReport` contracts in `ctxhelm-core`.
2. Add compiler aggregation from `HistoricalEvalReport` and
   `PolicyQualityReport`.
3. Add `ctxhelm eval health`.
4. Add Markdown renderer, docs, and smoke script.
5. Run focused tests, smoke, docs checks, CLI help, and workspace tests.

## Verification

- `cargo fmt --all`
- Focused retrieval health tests
- `bash scripts/smoke-retrieval-health.sh`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm -- eval health --help`
- `cargo test --workspace`

## Non-goals

- No raw prompt/source/session transcript persistence.
- No hosted dashboard.
- No graph-specific policy changes yet.
