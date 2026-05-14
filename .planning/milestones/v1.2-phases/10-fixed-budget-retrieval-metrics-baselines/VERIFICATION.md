---
phase: 10
status: passed
verified: 2026-05-14
---

# Verification: Phase 10 Fixed-Budget Retrieval Metrics & Baselines

## Verdict

Passed.

Phase 10 added no-context baseline comparison, budget-aligned lift metrics, token ROI rows for brief/standard/deep packs, and JSON/Markdown report coverage without storing source snippets, prompt text, or commit subjects.

## Requirements

| Requirement | Status | Evidence |
|-------------|--------|----------|
| METR-01 | Passed | Existing file/test recall, useful-target precision, and missing-label summaries remain in `HistoricalEvalReport` |
| METR-02 | Passed | `EvalComparison` now includes `combined`, `lexicalBaseline`, and `noContextBaseline` under the same K budget |
| METR-03 | Passed | `signalAblations` remains part of historical and benchmark reports |
| METR-04 | Passed | JSON and Markdown outputs include the new metrics and source-free tests assert no source/prompt text |
| METR-05 | Passed | Effective filters, refs, suite config metadata, K budget, and repo IDs remain in reports |
| ROI-01 | Passed | `tokenRoi` reports useful targets per 1k estimated tokens for brief, standard, and deep |
| ROI-02 | Passed | `largerPackAddsLittleValue` marks larger packs that add no new useful target labels |

## Commands

```bash
cargo check --workspace
cargo test -p ctxpack-compiler ranking_metrics_historical_eval_reports_fixed_budget_without_source_text -- --nocapture
cargo test -p ctxpack --test cli_compat search_related_tests_dependencies_and_eval_history_emit_json_shapes -- --nocapture
cargo test -p ctxpack-compiler -- --nocapture
cargo test -p ctxpack historical_eval_report_renders_source_free_metrics -- --nocapture
cargo test -p ctxpack --test cli_compat eval_benchmark_runs_named_suite_source_free -- --nocapture
```

## Notes

The no-context baseline is intentionally zero-file. Editor-open-file or current-buffer anchor baselines require integration trace data and are deferred until those traces are available.
