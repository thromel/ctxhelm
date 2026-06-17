# Phase 325: Agent-Run Efficiency Status Verdict

## Goal

Make agent-run proof distinguish reliability gains from efficiency gains without
requiring maintainers to manually interpret read-efficiency counters.

## Change

- `scripts/check-agent-run-proof.py` now emits a derived
  `metrics.efficiencyStatus` object in source-free proof-check summaries.
- `ctxhelm inspector proof` renders a dedicated read-efficiency verdict.
- `ctxhelm eval agent-run` includes the same status in human-readable
  read-efficiency output.
- Release and feedback docs mention the `efficiencyStatus` claim boundary.

## Verdict Semantics

The status is conservative:

- `reliability_and_efficiency_improved`: target-read coverage improved and the
  efficient ctxhelm lane did not add reads or irrelevant reads.
- `reliability_improved_with_read_overhead`: target-read coverage improved, but
  the efficient ctxhelm lane added reads or irrelevant reads.
- `efficiency_improved_without_reliability_lift`: reads improved without a
  target-read coverage lift.
- `no_efficiency_lift`: neither reliability nor read efficiency improved.
- `not_reported` / `insufficient_metrics`: proof lacks enough source-free
  counters for the verdict.

## Phase 324 Recheck

The Phase 324 Codex breadth-suite proof now reports:

```json
{
  "status": "reliability_improved_with_read_overhead",
  "reliabilityImproved": true,
  "efficiencyImproved": false,
  "efficiencyPromotionAllowed": false,
  "extraReadFileCount": 5,
  "extraIrrelevantReadCount": 3,
  "recoveredTargetReadCount": 2,
  "targetReadCoverageDelta": 0.14583333333333337
}
```

## Research Boundary

Allowed claim:

```text
ctxhelm improved Codex target-read reliability while making read overhead
explicit and non-promotable as efficiency evidence.
```

Not allowed yet:

```text
ctxhelm improves read efficiency in real Codex runs.
```

The next R&D slice should reduce the `reliability_improved_with_read_overhead`
case toward `reliability_and_efficiency_improved`.
