# Phase 339: Agent-Run Proof Run Metrics

## Summary

Single-run agent-run proof summaries now expose command/tool-call audit metrics
that were already available in suite summaries.

## Change

`scripts/check-agent-run-proof.py --workflow run --format json` now includes:

- `metrics.commandExecutionDelta`
- `metrics.ctxhelmToolCallsObserved`

This keeps standalone accepted reports inspectable without requiring a suite
aggregate wrapper.

## Acceptance

- The Phase 322 single-run fixture passes the proof checker.
- The JSON summary reports `metrics.commandExecutionDelta = 2`.
- The JSON summary reports `metrics.ctxhelmToolCallsObserved = true`.
- Release and feedback docs name the run-level audit fields.
- The release-doc checker requires the documented run-level fields.

## Non-goals

- No retrieval algorithm changes.
- No new source text, prompts, transcripts, MCP traffic, or command output in
  proof summaries.
- No weakening of suite-level aggregate consistency checks.
