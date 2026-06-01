# Phase 31 Plan: Policy Quality Reports

## Goal

Turn source-free feedback events into local policy quality reports that show whether ctxhelm recommendations were useful.

## Requirements

- LEARN-05
- LEARN-06
- LEARN-07
- LEARN-08

## Implementation

- Add `PolicyQualityReport` contracts for source-free aggregate metrics.
- Compare recommended files/tests against read, edited, tested, and corrected feedback.
- Compute context precision, read precision, edit recall proxy, validation coverage, correction rate, repeated missing-file families, signal contributions, and token ROI.
- Add `ctxhelm eval policy report` with Markdown and JSON renderers.
- Add focused tests proving reports remain source-free and label low-sample evidence.

## Validation

- `cargo fmt --all --check`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-index feedback -- --test-threads=1 --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm eval_policy_and_outcome_reports_are_source_free --test cli_compat -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test --workspace`
