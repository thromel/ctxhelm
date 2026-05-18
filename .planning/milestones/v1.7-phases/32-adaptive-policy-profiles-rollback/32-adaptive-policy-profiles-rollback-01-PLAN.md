# Phase 32 Plan: Adaptive Policy Profiles & Rollback

## Goal

Add explicit, source-free local retrieval-policy profiles that can be proposed, inspected, applied, disabled, and rolled back.

## Requirements

- LEARN-09
- LEARN-10
- LEARN-11
- LEARN-12

## Implementation

- Add `RetrievalPolicyProfile`, signal weight, safety floor, and action report contracts.
- Generate candidate profiles from policy quality evidence without mutating active behavior by default.
- Persist profiles locally under `CTXPACK_HOME` with candidate, active, disabled, and rolled-back states.
- Enforce conservative safety floors for anchor, lexical, and related-test signals.
- Add `ctxpack eval policy tune|list|apply|disable|rollback` with Markdown and JSON output.

## Validation

- `cargo fmt --all --check`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-index feedback -- --test-threads=1 --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack eval_policy_and_outcome_reports_are_source_free --test cli_compat -- --nocapture`
- `bash scripts/smoke-feedback.sh`
- `CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test --workspace`
