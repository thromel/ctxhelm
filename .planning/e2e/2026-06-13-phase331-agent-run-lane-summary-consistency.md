# Phase 331: Agent-Run Lane Summary Consistency

## Goal

Make saved Codex agent-run proof reject stale lane-summary metrics, not only
stale aggregate counts and lane names.

## Change

`scripts/check-agent-run-proof.py` now derives `aggregate.laneSummaries[*]`
metrics from nested `tasks[*].lanes[*]` records:

- summed read, target-read, irrelevant-read, command, required-call, evidence,
  missed-target, and discovered-only counters
- task, passed, comparison-eligible, failure, and rate-limit counters
- average target-read and target coverage
- target-read precision, irrelevant-read rate, and reads per target read
- `readRoleCounts` and `missedTargetRoleCounts`

The JSON proof-check artifact now reports
`aggregateConsistency.strictLaneSummaryMetricChecks`,
`aggregateConsistency.checkedLaneSummaryCount`, and
`aggregateConsistency.checkedLaneSummaryMetricCount`.

## Why

Phase 330 closed top-level aggregate count and lane-name drift. The remaining
summary-level gap was numeric: a stale `aggregate.laneSummaries[*]` entry could
keep old read-efficiency or coverage values while nested task lanes had changed.
Phase 331 makes those summary metrics derived from the source-free nested task
records during release proof validation.

## Validation Plan

- Direct positive `check-agent-run-proof.py` run against the saved Phase 322
  Codex breadth-suite report.
- Negative stale `aggregate.laneSummaries[*].readFileCount` mutation.
- Focused `release_packaging` contract test.
- Release-doc smoke.
- Full workspace tests before push.
