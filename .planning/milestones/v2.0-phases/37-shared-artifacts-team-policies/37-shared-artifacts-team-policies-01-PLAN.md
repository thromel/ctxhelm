# Phase 37 Plan: Shared Artifacts & Team Policies

## Goal

Let maintainers export, inspect, and import source-free shared artifact
manifests and initialize local team privacy policy templates.

## Scope

- Add shared artifact contracts for context cards, benchmark reports, policy
  profiles, feedback summaries, proof reports, workspace manifests, and team
  policies.
- Add team privacy policy contracts and reports covering local indexing,
  artifact export, optional cloud allowances, source snippet allowances, and
  secret redaction defaults.
- Add index-layer artifact manifest export/import/inspect APIs.
- Add CLI commands under `ctxpack workspace artifacts ...` and
  `ctxpack workspace policy ...`.
- Add docs, CLI tests, and release-gate smoke coverage.

## Verification

- `cargo test -p ctxpack-core shared`
- `cargo test -p ctxpack-index shared`
- `cargo test -p ctxpack workspace --test cli_compat`
- `scripts/smoke-shared-artifacts.sh`

