# Phase 207 - Agent-Run Lane Comparability

## Goal

Prevent paired real-agent reports from claiming ctxhelm lift or weakness when a
ctxhelm-assisted lane failed before using the required ctxhelm tools.

## Change

- `scripts/e2e-agent-run.sh` now records per-lane required ctxhelm calls,
  observed required calls, missing required calls, `ctxhelmCallCompliance`,
  `evaluationStatus`, and `evaluationEligible`.
- Required calls are mode-specific:
  - `baseline`: none
  - `ctxhelm-plan`: `prepare_task`
  - `ctxhelm-brief`: `prepare_task`, `get_pack`
- Single-run reports now include `comparisonEligible`, `baselineEligible`,
  `comparableCtxhelmLaneCount`, `missingRequiredCtxhelmCallsObserved`, and
  `missingRequiredCtxhelmCalls`.
- Suite reports aggregate comparison-eligible task count, comparable ctxhelm lane
  count, missing required-call observations, and per-lane required/observed/missing
  call counts.
- Outcome claims now use `insufficient_comparable_lanes` when no baseline plus
  ctxhelm-assisted lane is defensibly comparable.
- `ctxhelm eval agent-run` renders the new comparability fields in single-run
  and suite markdown reports.

## Validation

- `bash -n scripts/e2e-agent-run.sh` passed.
- `cargo fmt --check` passed after formatting.
- `cargo test -p ctxhelm eval_agent_run --locked -- --nocapture` passed.
- `cargo test -p ctxhelm agent_run_e2e_script_contract --locked -- --nocapture` passed.
- Skipped smoke report `/tmp/ctxhelm-phase208-skipped.json` produced
  `outcomeClaim = insufficient_comparable_lanes`, `comparisonEligible = false`,
  and missing required calls for `ctxhelm-plan` and `ctxhelm-brief`.

## Interpretation

This phase does not claim a retrieval-quality or agent-consumption lift. It
hardens the R&D proof boundary so future Claude Code/Codex paired runs can
distinguish:

- ctxhelm was used and produced comparable evidence;
- ctxhelm was configured but the agent skipped a required tool call;
- the lane failed before the brief-pack path was tested.
