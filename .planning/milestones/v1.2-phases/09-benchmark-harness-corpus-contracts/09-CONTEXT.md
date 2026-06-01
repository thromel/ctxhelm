# Phase 9: Benchmark Harness & Corpus Contracts - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Mode:** Auto-generated infrastructure context

<domain>
## Phase Boundary

Build the v1.2 benchmark foundation: named source-free benchmark suite contracts, multi-repo execution, reproducibility metadata, privacy/skipped-path reporting, and setup documentation. This phase should not attempt deeper metrics, token ROI, trend comparison, or product proof messaging beyond what is needed to establish the corpus runner.

</domain>

<decisions>
## Implementation Decisions

### Use Existing Historical Eval

The benchmark suite should wrap the existing `evaluate_historical_commits` path rather than introducing a separate evaluator. This keeps Phase 9 scoped to corpus contracts and avoids duplicating retrieval logic.

### JSON Suite Contract First

Use a JSON suite file because the workspace already depends on `serde_json` and uses JSON for CLI/MCP contracts. Avoid adding TOML/YAML dependencies before the storage milestone.

### Source-Free Reports

Reports may include suite names, repo labels, repo IDs, revision metadata, safe path labels, counts, and metrics. Reports must not include source snippets, prompt text, or commit subjects.

### Future Role Filters

Role filters are part of the suite contract and source-free metadata now. Deeper role-filtered metric segmentation can be expanded in Phase 10/11.

</decisions>

<code_context>
## Existing Code Insights

- `crates/ctxhelm-compiler/src/eval.rs` owns `HistoricalEvalOptions`, `HistoricalEvalReport`, `evaluate_historical_commits`, lexical baseline comparison, signal ablations, and retrieval gap summaries.
- `crates/ctxhelm/src/main.rs` owns `ctxhelm eval history`, Markdown rendering, and CLI JSON output.
- `crates/ctxhelm-index/src/git.rs` owns historical commit sampling and safe changed-path labels.
- `crates/ctxhelm/tests/cli_compat.rs` is the right location for binary-level CLI compatibility checks.
- `docs/` already contains install, release, quickstart, troubleshooting, and agent setup docs.

</code_context>

<specifics>
## Specific Ideas

- Add `BenchmarkSuiteConfig`, `BenchmarkRepoConfig`, `BenchmarkSuiteReport`, and `BenchmarkRepoReport`.
- Add `ctxhelm eval benchmark --config <file> --format markdown|json`.
- Resolve relative repo paths against the suite file location.
- Preserve source-free privacy in JSON and Markdown output.
- Add documentation for RefactoringMiner-style local benchmark setup.

</specifics>

<deferred>
## Deferred Ideas

- Token ROI and no-context baseline details belong in Phase 10.
- Gap taxonomy and regression comparisons belong in Phase 11.
- Product proof report and release-gate benchmark smoke belong in Phase 12.

</deferred>
