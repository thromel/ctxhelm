# Phase 328: Agent-Run Task-Lane Proof

## Goal

Prevent a saved real-agent outcome report from passing release validation when
only aggregate fields are clean but a nested task lane contains bad evidence.

Earlier proof hardening made the saved report fresh against the current runner,
suite, ctxhelm version, and Codex client identity. Phase 328 makes the checker
look inside each task and lane instead of trusting only aggregate summaries.

## Change

`scripts/check-agent-run-proof.py` now validates every task in a suite report:

```text
tasks[*].comparison.comparisonEligible == true
tasks[*].comparison strict boundary fields == false
tasks[*].lanes[*].status == passed
tasks[*].lanes[*].clientExitStatus == 0
tasks[*].lanes[*].metrics.forbiddenCommandCount == 0
tasks[*].lanes[*].metrics.missingRequiredCtxhelmCallCount == 0
tasks[*].lanes[*].metrics.invalidRequiredCtxhelmCallCount == 0
tasks[*].lanes[*].forbiddenCommands is empty
tasks[*].lanes[*].clientFailure is null
tasks[*].lanes[*].rateLimit is null
```

For ctxhelm lanes, the checker also requires:

```text
targetReadCoverage >= release floor
ctxhelmEvidenceMissedTargetCount == 0
ctxhelmEvidenceOnlyTargetCount == 0
missedTargetCount == 0
targetDiscoveredOnlyCount == 0
requiredCtxhelmCalls is non-empty
observedRequiredCtxhelmCallCount >= requiredCtxhelmCallCount
ctxhelmToolCallCount >= requiredCtxhelmCallCount
ctxhelmEvidenceMisses is empty or null
ctxhelmEvidenceOnlyTargets is empty
ctxhelmUnderReadTargets is empty or null
```

The JSON audit artifact now records:

```text
taskLaneChecks.strictTaskLaneChecks
taskLaneChecks.taskLaneCount
taskLaneChecks.ctxhelmTaskLaneCount
```

## Validation

The committed Phase 322 report still passes with:

```text
taskLaneChecks.taskLaneCount = 20
taskLaneChecks.ctxhelmTaskLaneCount = 16
```

The focused Rust contract now creates a stale task-lane fixture by setting
`tasks[0].lanes[1].metrics.ctxhelmEvidenceOnlyTargetCount = 1` while leaving
the aggregate fields unchanged. The checker rejects that fixture.

```bash
cargo test -p ctxhelm --test release_packaging \
  agent_run_proof_checker_accepts_phase322_and_rejects_regression --locked
```
