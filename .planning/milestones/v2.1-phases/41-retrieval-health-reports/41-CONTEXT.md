# Phase 41: Retrieval Health Reports - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning
**Mode:** Autonomous inline execution

<domain>
## Phase Boundary

Phase 41 adds source-free retrieval health reporting. It should aggregate the
evidence ctxpack already produces: historical eval recall, token ROI, retrieval
gap families, signal ablations, and feedback/policy quality.

This phase should not add a hosted analytics backend or raw session logging.
</domain>

<decisions>
## Implementation Decisions

1. Build the health report from `HistoricalEvalReport` plus
   `PolicyQualityReport`.
2. Keep the contract in `ctxpack-core` so CLI, inspector, and future MCP/UI
   surfaces share one shape.
3. Add `ctxpack eval health` as the CLI entry point.
4. Keep the report source-free with `sourceTextLogged: false`.
5. Use existing eval and feedback flows instead of adding a new storage schema.
</decisions>

<code_context>
## Existing Code Insights

- Historical eval already includes file/test recall, token ROI, signal
  ablations, and gap summaries.
- Feedback policy quality already includes context precision, validation
  coverage, correction rate, token ROI, repeated missing files, and signal
  contributions.
- CLI `eval history`, `eval policy report`, and `eval outcome compare` already
  expose adjacent pieces separately.
</code_context>

<specifics>
## Specific Ideas

- Add a compact `RetrievalHealthReport`.
- Add health metric rows, token ROI rows, signal contribution rows, gap family
  rows, and low-confidence flags.
- Add Markdown and JSON output through `ctxpack eval health`.
- Add docs/smoke coverage after the CLI path is stable.
</specifics>

<deferred>
## Deferred Ideas

- Inspector-side visual integration can be expanded later.
- Graph-specific health signals belong to Phase 42/43.
</deferred>
