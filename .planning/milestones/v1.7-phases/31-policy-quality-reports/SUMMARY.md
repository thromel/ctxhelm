# Phase 31 Summary: Policy Quality Reports

## Delivered

- Added source-free policy quality report contracts.
- Added local report computation over feedback JSONL events.
- Added report metrics for context precision, read precision, edit recall proxy, validation coverage, correction rate, repeated missing-file families, signal contributions, and token ROI.
- Added `ctxhelm eval policy report` in Markdown and JSON.
- Added source-free CLI compatibility coverage for policy reporting.

## Notes

Reports intentionally label low-sample evidence and do not store raw prompts, source snippets, terminal output, or model transcripts.
