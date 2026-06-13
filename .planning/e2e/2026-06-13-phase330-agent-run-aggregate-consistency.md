# Phase 330: Agent-Run Aggregate Consistency

## Goal

Make saved Codex agent-run proof harder to spoof by deriving top-level aggregate
claims from the nested source-free task and lane records.

## Change

`scripts/check-agent-run-proof.py` now recomputes these suite-level fields from
`tasks[*]`:

- task count
- comparison-eligible task count
- comparable ctxhelm lane count
- strict boundary booleans
- aggregate lane-summary names

The checker fails when `aggregate.*` counters or `aggregate.laneSummaries`
names drift from derived task comparisons. JSON audit output now includes an
`aggregateConsistency` block with derived counts and a
`matchesDerivedAggregates` proof flag.

## Why

Earlier phases made the saved report strict on identity, runner fingerprint,
suite fingerprint, current suite tasks, and nested lane records. The remaining
weakness was aggregate trust: a report could keep clean nested task data while
editing top-level aggregate counters or lane names. Phase 330 closes that gap
without rerunning live agents or storing prompts, transcripts, MCP traffic, raw
command output, or source text.

## Validation Plan

- Direct positive `check-agent-run-proof.py` run against the saved Phase 322
  Codex breadth-suite report with current runner and suite checks.
- Negative proof mutation for stale `aggregate.comparisonEligibleCount`.
- Negative proof mutation for stale `aggregate.laneSummaries[*].lane`.
- Focused `release_packaging` contract test.
- Release-doc smoke.
- Full workspace tests before push.
