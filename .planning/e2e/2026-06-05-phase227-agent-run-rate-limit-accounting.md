# Phase 227 Agent-Run Rate-Limit Accounting

## Goal

Rerun the Phase 225 five-lane Claude Code matrix after the Phase 226 retrieval fix and make sure unavailable-client reports do not look like agent-consumption failures.

## Change

- `scripts/e2e-agent-run.sh` now clears `ctxhelmEvidenceOnlyTargets` for non-evaluation-eligible lanes.
- Single-task under-read comparison now considers only evaluation-eligible ctxhelm lanes.
- Suite-level aggregation inherits the corrected task-level evidence-only signal and failed-lane evidence-only counts.

This preserves retrieval diagnostics: `ctxhelmEvidenceMissedTargets` can still report whether ctxhelm surfaced expected targets. It only removes consumption-style diagnostics from lanes where the client failed before a comparable read phase.

## Proof

Artifact: `.ctxhelm/e2e/phase227-agent-run-rate-limit-accounting.json`

Command:

```bash
CTXHELM_RUN_REAL_CLIENT=1 CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=120 \
  bash scripts/e2e-agent-run.sh \
    --repo /Users/romel/Documents/GitHub/ctxhelm \
    --task "Improve paired agent-run lane matrix" \
    --target-file scripts/e2e-agent-run.sh \
    --target-file docs/feedback.md \
    --output .ctxhelm/e2e/phase227-agent-run-rate-limit-accounting.json
```

Result:

- Claude Code `2.1.163` still rate-limited all five lanes.
- `comparison.outcomeClaim = "insufficient_comparable_lanes"`.
- `comparison.ctxhelmEvidenceMissesObserved = false`.
- `comparison.ctxhelmEvidenceOnlyTargetsObserved = false`.
- All rate-limited lanes have `metrics.ctxhelmEvidenceOnlyTargetCount = 0`.
- The only recommended action is `retry_real_client_when_available`.

## Boundary

This is report hardening and a failed-client rerun, not outcome-lift proof. A non-rate-limited five-lane run is still required before claiming ctxhelm improves real-agent behavior on this task.
