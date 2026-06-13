# Phase 332: Agent-Run Retry And Read-Efficiency Consistency

## Goal

Make saved Codex agent-run proof reject stale retry-cost and read-efficiency
aggregate claims.

## Change

`scripts/check-agent-run-proof.py` now derives:

- `aggregate.retryCost` from each task's `comparison.retryCost`
- `aggregate.readEfficiency` from `aggregate.laneSummaries`

The checker rejects stale aggregate retry overhead, evidence-only target,
target-read coverage, recovered-target, extra-read, read-precision, and
irrelevant-read-rate metrics. JSON proof-check output now includes:

- `aggregateConsistency.strictRetryCostConsistencyChecks`
- `aggregateConsistency.checkedRetryCostMetricCount`
- `aggregateConsistency.strictReadEfficiencyConsistencyChecks`
- `aggregateConsistency.checkedReadEfficiencyMetricCount`

## Why

Phases 330 and 331 made task, lane, and lane-summary proof strict. The remaining
aggregate drift surface was outcome-adjacent: retry-cost and read-efficiency
claims could become stale even when nested task records and lane summaries were
valid. Phase 332 closes that source-free proof gap without rerunning live
clients or storing prompts, transcripts, command output, MCP traffic, or source
text.

## Validation Plan

- Direct positive `check-agent-run-proof.py` run against the saved Phase 322
  Codex breadth-suite report.
- Negative stale `aggregate.retryCost.avgReadFilesAfterRetry` mutation.
- Negative stale `aggregate.readEfficiency.extraReadFileCount` mutation.
- Focused `release_packaging` contract test.
- Release-doc smoke.
- Full workspace tests before push.
