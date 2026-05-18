# Phase 44 Plan: Agent Preview & Inspector Release Gates

**Created:** 2026-05-18
**Status:** In Progress

## Scope

Implement AGPREV-01 through AGPREV-04.

## Steps

1. Add source-free agent preview contracts.
2. Add compiler preview builder that reuses plan/pack/resource metadata.
3. Add `ctxpack agent preview` with JSON and Markdown output.
4. Add docs and smoke coverage.
5. Wire the smoke into release docs, release-gate script, and packaging tests.
6. Run formatting, smoke tests, release-doc checks, CLI help, and workspace tests.

## Verification

- `cargo fmt --all --check`
- `bash scripts/smoke-agent-preview.sh`
- `bash scripts/check-release-docs.sh`
- `bash -n scripts/release-gate.sh`
- `cargo run -p ctxpack -- agent preview --help`
- `cargo run -p ctxpack -- --help`
- `cargo test --workspace`
