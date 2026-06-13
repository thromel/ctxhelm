# Phase 334: Agent-Run Outcome Routing Consistency

## Goal

Make saved Codex agent-run proof reject stale outcome labels and stale R&D
routing actions.

## Change

`scripts/check-agent-run-proof.py` now derives:

- `aggregate.outcomeClaim`
- `aggregate.recommendedResearchActions`

from source-free `tasks[*].comparison` aggregates and strict boundary flags.
The JSON proof-check artifact reports:

- `aggregateConsistency.strictOutcomeRoutingChecks`
- `aggregateConsistency.derivedOutcomeClaim`
- `aggregateConsistency.checkedRecommendedResearchActionCount`

## Why

Earlier phases made source-free counts, lane summaries, retry cost, read
efficiency, and comparison aggregate deltas strict. The remaining direct
claim-routing fields were the top-level outcome label and recommended research
action list. Phase 334 closes that drift surface without rerunning live clients
or storing prompts, transcripts, MCP traffic, command output, or source text.

## Validation Plan

- Direct positive `check-agent-run-proof.py` run against the saved Phase 322
  Codex breadth-suite report.
- Negative stale `aggregate.outcomeClaim` mutation.
- Negative stale `aggregate.recommendedResearchActions` mutation.
- Focused `release_packaging` contract test.
- Release-doc smoke.
- Full workspace tests before push.
