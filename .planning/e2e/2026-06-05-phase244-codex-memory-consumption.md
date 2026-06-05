# Phase 244 - Codex Memory Consumption Fix

## Summary

Phase 244 fixes the real-agent memory-consumption gap exposed by the VeriSchema Codex run. Earlier runs showed ctxhelm surfaced `schema_agent/core/models.py` through selected memory evidence, but Codex did not consume the file with native read commands. The root cause was twofold:

- selected memory source links could remain evidence-only when ordinary lexical targets filled the target-file budget;
- the Codex e2e MCP wrapper did not explicitly carry `CTXHELM_HOME`, so the real MCP server could start without the seeded memory store.

## Changes

- Promoted up to 3 fresh/approved selected-memory source links into `targetFiles` as first-read targets.
- Added `selected_memory_initial_read` attribution and a `selected_memory_initial_read_promoted` diagnostic.
- Added a confidence floor for selected-memory first-read targets so agents do not sort them behind later lexical paths.
- Added an MCP `Native read queue` with concrete read commands above the `ContextPlan` JSON.
- Baked `CTXHELM_HOME` into the generated Codex MCP wrapper when the e2e script is run with a custom memory home.

## Proof

Artifact: `.ctxhelm/e2e/phase244-agent-run-codex-memory-consumption.json`

Rendered result:

- Status: `passed`
- Client: `codex-cli 0.137.0`
- Comparison eligible: `true`
- Best lane: `ctxhelm-brief`
- Outcome claim: `ctxhelm_improved`
- Evidence-only targets observed: `false`
- Missing required ctxhelm calls: `none`
- Invalid required ctxhelm calls: `none`
- Forbidden tool calls: `false`
- Client failures: `false`
- Source text logged: `false`

Target consumption:

- Baseline read target coverage: `1.00`
- ctxhelm-plan read target coverage: `1.00`
- ctxhelm-brief read target coverage: `1.00`
- ctxhelm-standard read target coverage: `1.00`
- ctxhelm-memory read target coverage: `1.00`

Efficiency:

- Baseline read files: `18`
- Best ctxhelm lane read files: `5`
- Read-file delta: `13`
- Baseline irrelevant reads: `17`
- Best ctxhelm lane irrelevant reads: `4`
- Irrelevant-read delta: `13`
- Command-execution delta: `16`

## Interpretation

This is a real R&D improvement, not just guidance hardening. Phase 240-243 demonstrated that ctxhelm could surface the memory target but the agent did not consume it. Phase 244 verifies that, after selected-memory promotion and MCP environment propagation, Codex consumes `schema_agent/core/models.py` in all ctxhelm lanes while preserving source-free evidence boundaries.

The comparison delta is efficiency-focused because the Phase 244 baseline also found the target in this run. The corrected behavior still matters: ctxhelm removes the prior evidence-only gap and cuts the best lane from 18 read files to 5 while preserving target-read coverage.
