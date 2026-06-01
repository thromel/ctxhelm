# Phase 34 Plan: Feedback Docs, External Smoke, and Release Gates

## Goal

Document feedback learning and wire deterministic release checks that prove v1.7 stays local, source-free, measurable, and backward-compatible.

## Requirements

- LEARN-17
- LEARN-18
- LEARN-19
- LEARN-20

## Implementation

- Add `docs/feedback.md` covering feedback events, reports, profiles, rollback, privacy guarantees, outcome comparison, and anti-patterns.
- Add `scripts/smoke-feedback.sh` to exercise feedback ingestion, policy reports, candidate tuning, apply/rollback metadata, and outcome comparison.
- Wire feedback docs and smoke into release documentation checks and release gate.
- Preserve existing prepare-task, get-pack, MCP, memory, semantic, precision, and storage behavior by default.
- Keep external large-history smoke claims bounded unless a fixed sample is actually run.

## Validation

- `bash scripts/smoke-feedback.sh`
- `bash scripts/check-release-docs.sh`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo run -p ctxhelm -- --help`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test --workspace`
