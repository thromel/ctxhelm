# Phase 337: Agent-Run Proof Privacy Boundary Summary

## Goal

Make saved Codex agent-run proof audit summaries unambiguous about factual
privacy and boundary values versus pass/fail checks.

## Change

`scripts/check-agent-run-proof.py` now emits:

- factual `privacyStatus` values copied from the saved report
- `privacyChecks` pass/fail fields for local-only and strict-false source-free
  privacy checks
- factual `boundaryStatus` values copied from aggregate or comparison fields
- `boundaryChecks` pass/fail fields for strict-false boundary checks

## Why

Earlier summaries used `privacyStatus.sourceTextLogged: true` to mean "the
source-text-logged check passed because the report value was false." That is
easy to misread and weakens the audit artifact for maintainers, release notes,
and downstream automation. Phase 337 keeps the proof source-free while making
facts and checks explicit.

## Validation Plan

- Direct JSON proof-check run against the saved Phase 322 Codex breadth-suite
  report.
- Focused `release_packaging` contract test asserting factual privacy and
  boundary values plus check pass fields.
- Release-doc smoke.
- Full workspace tests before push.
