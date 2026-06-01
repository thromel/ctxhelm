# Phase 41 Summary: Retrieval Health Reports

**Completed:** 2026-05-18
**Status:** Complete

## Delivered

- Added `RetrievalHealthReport` and child source-free contracts.
- Added compiler aggregation from `HistoricalEvalReport` and
  `PolicyQualityReport`.
- Added `ctxhelm eval health` with Markdown and JSON output.
- Added retrieval-health documentation.
- Added `scripts/smoke-retrieval-health.sh` and wired it into release docs and
  the release gate.
- Added low-confidence flags for weak file recall, test recall, validation
  coverage, and high correction rate.

## Verification

- `cargo fmt --all`
- `bash scripts/smoke-retrieval-health.sh`
- `bash scripts/check-release-docs.sh`
- `bash -n scripts/release-gate.sh`
- `bash -n scripts/smoke-retrieval-health.sh`
- `cargo run -p ctxhelm -- eval health --help`
- `cargo test -p ctxhelm release_gate_script_contract -- --nocapture`
- `cargo test -p ctxhelm release_docs_check_passes -- --nocapture`
- `cargo test --workspace`

All commands passed using `CARGO_TARGET_DIR=/tmp/ctxhelm-target-phase39` where
applicable.

## Next

Phase 42 can add source-free graph neighborhoods and communities that feed both
the inspector and retrieval-health diagnostics.
