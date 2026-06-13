# Phase 333: Agent-Run Comparison Aggregate Consistency

## Goal

Make saved Codex agent-run proof reject stale top-level comparison aggregate
metrics.

## Change

`scripts/check-agent-run-proof.py` now derives these aggregate fields from
`tasks[*].comparison`:

- `targetReadCoverageDeltaAverage`
- `targetCoverageDeltaAverage`
- `readFileDeltaSum`
- `irrelevantReadDeltaSum`
- `commandExecutionDeltaSum`
- `ctxhelmToolCallsObserved`

The JSON proof-check artifact now reports
`aggregateConsistency.strictComparisonAggregateChecks` and
`aggregateConsistency.checkedComparisonAggregateMetricCount`.

## Why

Phases 330-332 made saved agent-run proof strict for suite identity, nested task
lanes, lane summaries, retry cost, and read efficiency. The remaining direct
top-level drift class was comparison aggregate metrics. Phase 333 closes that
gap without rerunning live clients or storing prompts, transcripts, MCP traffic,
command output, or source text.

## Validation Plan

- Direct positive `check-agent-run-proof.py` run against the saved Phase 322
  Codex breadth-suite report.
- Negative stale `aggregate.targetReadCoverageDeltaAverage` mutation.
- Negative stale `aggregate.ctxhelmToolCallsObserved` mutation.
- Focused `release_packaging` contract test.
- Release-doc smoke.
- Full workspace tests before push.
