# Phase 53 Summary

## Completed

- Added source-free signal-only ranking snapshots to historical commit eval rows.
- Added paired baseline analysis contracts for default, lexical, no-context,
  semantic-only, graph-only, history-only, test-only, memory-only,
  feedback-weighted, and ablation variants.
- Added thresholded verdicts: `lift`, `neutral`, `regression`, and
  `insufficient_evidence`.
- Added lexical delta/status so lexical parity and regression are visible.
- Added token ROI, validation coverage, signal saturation, grouped retrieval
  gaps, runtime, and local-only privacy status to paired reports.
- Added `ctxhelm eval baselines` with Markdown and JSON output.
- Added compiler and CLI regression tests for the new report.
- Documented the paired baseline workflow in `docs/paired-baselines.md` and
  linked it from architecture and benchmarking docs.

## Notes

- `feedback_weighted` is intentionally `insufficient_evidence` until feedback
  labels are joined with fixed-corpus candidate rows in the learned-policy
  phase.
- Signal-only variants filter the same source-free candidate set as default
  ctxhelm ranking. They are diagnostic controls, not independently trained
  rankers.

