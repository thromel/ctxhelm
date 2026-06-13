# Phase 336: Agent-Run Proof Summary Metrics

## Goal

Make saved Codex agent-run proof audit summaries expose the same comparison
delta field names that the checker validates.

## Change

`scripts/check-agent-run-proof.py` now emits:

- `metrics.targetReadCoverageDeltaAverage`
- `metrics.targetCoverageDeltaAverage`

in JSON proof-check summaries. It no longer emits stale shortened
`targetReadCoverageDeltaAvg` or `targetCoverageDeltaAvg` aliases, which did not
exist in the validated aggregate report and therefore rendered as `null`.

## Why

The checker already derived and validated the aggregate comparison fields from
`tasks[*].comparison`, but the audit summary exposed the wrong names. That made
the release artifact less useful for humans and downstream automation. Phase
336 aligns the summary contract with the validated source-free aggregate
contract.

## Validation Plan

- Direct JSON proof-check run against the saved Phase 322 Codex breadth-suite
  report.
- Focused `release_packaging` contract test asserting non-null delta metrics
  under the exact validated names.
- Release-doc smoke.
- Full workspace tests before push.
