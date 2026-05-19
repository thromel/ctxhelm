# Phase 50: Fixed Benchmark Corpus & RefactoringMiner Regression Suite - Context

**Gathered:** 2026-05-19
**Status:** Ready for planning
**Mode:** Auto-generated infrastructure context

<domain>
## Phase Boundary

This phase turns the existing benchmark suite JSON into a v2.3 fixed corpus manifest and locks the first RefactoringMiner regression suite with source-free baseline metadata.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Extend the existing benchmark suite contract instead of creating a separate parser.
- Preserve backward compatibility for older benchmark suite JSON.
- Keep all corpus and baseline data source-free.
- Treat RefactoringMiner as a required optional external proof target, not as a universal product claim.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ctxpack-compiler/src/eval.rs` already owns benchmark suite config, historical eval reports, product proof, and comparison reports.
- `docs/benchmarking.md` already documents suite JSON and privacy boundaries.
- `.planning/e2e/2026-05-19-refactoringminer-full-e2e.md` records the current RefactoringMiner Recall@10 and lexical baseline evidence.

### Established Patterns
- Benchmark artifacts stay source-free and local-only.
- JSON contracts use camelCase serde fields.
- Existing suites should remain valid through defaulted new fields.

### Integration Points
- `ctxpack eval benchmark --config ...`
- `ctxpack eval proof --config ...`
- Release gates can opt into external benchmark proof through `CTXPACK_BENCHMARK_CONFIG`.

</code_context>

<specifics>
## Specific Ideas

- Add `.ctxpack/benchmarks/refactoringminer-v23.json` with the May 19 baseline.
- Surface manifest version, corpus ID, privacy label, revision range ID, and baseline deltas in reports.

</specifics>

<deferred>
## Deferred Ideas

- Parallel execution, cached eval reuse, and feature exports belong to later v2.3 phases.

</deferred>
