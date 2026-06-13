# Phase 338: Agent-Run Proof Comparison Metrics

## Goal

Make saved Codex agent-run proof audit summaries expose every validated
top-level comparison aggregate.

## Change

`scripts/check-agent-run-proof.py` now includes these fields in suite proof
summary `metrics`:

- `targetReadCoverageDeltaAverage`
- `targetCoverageDeltaAverage`
- `readFileDeltaSum`
- `irrelevantReadDeltaSum`
- `commandExecutionDeltaSum`
- `ctxhelmToolCallsObserved`

## Why

Phase 333 made those comparison aggregates strict by deriving them from
`tasks[*].comparison`. Later audit-summary fixes corrected coverage-delta names,
but the JSON summary still omitted command delta and ctxhelm tool-call
observation. Phase 338 aligns the audit surface with the validated contract so
release artifacts and downstream automation can inspect the full comparison
aggregate proof without rereading the source report.

## Validation Plan

- Direct JSON proof-check run against the saved Phase 322 Codex breadth-suite
  report.
- Focused `release_packaging` contract test asserting command delta and ctxhelm
  tool-call observation appear in summary metrics.
- Release-doc smoke.
- Full workspace tests before push.
