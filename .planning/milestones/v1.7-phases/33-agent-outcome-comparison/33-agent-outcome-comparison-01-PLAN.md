# Phase 33 Plan: Agent Outcome Comparison

## Goal

Compare plan-only, brief, standard, and deep pack outcomes from source-free feedback.

## Requirements

- LEARN-13
- LEARN-14
- LEARN-15
- LEARN-16

## Implementation

- Add `AgentOutcomeComparisonReport` and `BudgetOutcome` contracts.
- Aggregate feedback by pack budget, including plan-only events with no pack budget.
- Report pass rate, blocked rate, correction rate, validation coverage, average context size, and useful target files per 1k estimated tokens.
- Emit low-sample and missing-validation warnings before claiming lift.
- Add `ctxpack eval outcome compare` with Markdown and JSON output.

## Validation

- `cargo fmt --all --check`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-index feedback -- --test-threads=1 --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack eval_policy_and_outcome_reports_are_source_free --test cli_compat -- --nocapture`
- `bash scripts/smoke-feedback.sh`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test --workspace`
