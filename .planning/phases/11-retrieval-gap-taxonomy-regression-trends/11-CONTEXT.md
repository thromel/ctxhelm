# Phase 11: Retrieval Gap Taxonomy & Regression Trends - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Mode:** Auto-generated infrastructure context

<domain>
## Phase Boundary

Turn benchmark misses into source-free failure families and add a regression comparison path for benchmark JSON reports. This phase should make repeated retrieval misses actionable for future milestones without storing source snippets, prompt text, or commit subjects.

</domain>

<decisions>
## Implementation Decisions

### Extend Existing Gap Summaries

Build on `RetrievalGapSummary` rather than introducing a separate taxonomy report. Existing historical eval and benchmark suite reports already carry these summaries.

### Compare Benchmark Reports, Not Source Repos

Regression trend comparison should consume source-free benchmark JSON reports. This keeps the comparison reproducible and avoids rereading source while comparing runs.

### Thresholds Are Reported Source-Free

Threshold checks should produce pass/fail booleans in the comparison report. The CLI remains report-oriented; release-gate hard failure belongs in Phase 12.

</decisions>

<code_context>
## Existing Code Insights

- `crates/ctxpack-compiler/src/eval.rs` owns gap summary construction, benchmark suite reports, and typed eval contracts.
- `crates/ctxpack/src/main.rs` owns CLI subcommands and Markdown rendering.
- `docs/benchmarking.md` documents benchmark setup and interpretation.
- `crates/ctxpack/tests/cli_compat.rs` is the right place for command-level compare coverage.

</code_context>

<specifics>
## Specific Ideas

- Add package, target status, and recommendation area to grouped retrieval gaps.
- Distinguish current reachable, renamed, deleted, and policy-excluded target labels.
- Add source-free benchmark report comparison with metric deltas and gap-family deltas.
- Add threshold checks for selected metrics such as `fileRecallAt10` and token ROI.

</specifics>

<deferred>
## Deferred Ideas

- Release-gate hard failure belongs to Phase 12.
- Storage-backed trend history belongs to v1.3.
- Semantic retrieval and parser precision fixes belong to v1.4/v1.5 after measured gap evidence.

</deferred>
