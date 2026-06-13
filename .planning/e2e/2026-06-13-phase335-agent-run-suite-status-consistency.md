# Phase 335: Agent-Run Suite Status Consistency

## Goal

Make saved Codex agent-run proof reject stale suite-envelope status and task
count fields.

## Change

`scripts/check-agent-run-proof.py` now derives:

- `suite.taskCount`
- `report.status`

from nested `tasks[*]` records, comparison eligibility, and strict boundary
flags. The JSON proof-check artifact reports:

- `suiteConsistency.strictSuiteStatusChecks`
- `suiteConsistency.suiteTaskCount`
- `suiteConsistency.derivedTaskCount`
- `suiteConsistency.derivedComparisonEligibleCount`
- `suiteConsistency.derivedBoundaryObserved`
- `suiteConsistency.derivedStatus`
- `suiteConsistency.matchesDerivedTaskCount`
- `suiteConsistency.matchesDerivedStatus`

## Why

Phases 330-334 made aggregate counts, lane summaries, retry/read-efficiency
metrics, comparison aggregates, and outcome routing strict. The remaining
suite-envelope drift surface was a stale top-level status or stale suite task
count that no longer matched the nested task records. Phase 335 closes that
surface without rerunning live clients or storing prompts, transcripts, MCP
traffic, command output, or source text.

## Validation Plan

- Direct positive `check-agent-run-proof.py` run against the saved Phase 322
  Codex breadth-suite report.
- Negative stale `suite.taskCount` mutation.
- Negative stale `report.status` mutation.
- Focused `release_packaging` contract test.
- Release-doc smoke.
- Full workspace tests before push.
