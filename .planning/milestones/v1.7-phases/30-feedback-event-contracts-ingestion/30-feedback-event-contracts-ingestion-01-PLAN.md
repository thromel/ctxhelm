# Phase 30 Plan: Feedback Event Contracts & Ingestion

## Goal

Add the first v1.7 feedback loop foundation: source-free agent-session feedback events that can be recorded, listed, and summarized locally.

## Requirements

- LEARN-01
- LEARN-02
- LEARN-03
- LEARN-04

## Implementation

- Add stable public `SessionFeedbackEvent`, `FeedbackOutcome`, and `FeedbackSummary` contracts in `ctxpack-core`.
- Add local JSONL feedback ingestion under `CTXPACK_HOME/repos/<repo-id>/feedback.jsonl`.
- Validate feedback labels so stored events remain source-free, single-line, repository-relative, and explicitly `sourceTextLogged=false`.
- Add CLI surface under `ctxpack eval feedback record|list|summary`.
- Add focused core, index, and CLI tests.

## Validation

- `cargo fmt --all --check`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-core feedback_event_public_json_shape_is_source_free -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-index feedback_events_ -- --test-threads=1 --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack eval_feedback_records_lists_and_summarizes_source_free_events --test cli_compat -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo run -p ctxpack -- --help`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test --workspace`
