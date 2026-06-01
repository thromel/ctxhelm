# Phase 51: Historical Eval Cache & Parallel Runner - Context

**Gathered:** 2026-05-19
**Status:** Ready for planning
**Mode:** Auto-generated infrastructure context

<domain>
## Phase Boundary

This phase makes large-history historical evals faster and more inspectable without changing ctxhelm's source-free privacy boundary. It builds on Phase 50's fixed benchmark corpus manifests.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Add source-free stored report reuse before recomputing expensive historical eval ranges.
- Keep cache keys tied to repo identity, eval options, refs, and an explicit cache schema version.
- Add deterministic commit-sample parallelism without changing final report ordering.
- Expose timing diagnostics in reports and CLI output.
- Do not persist source snapshots, prompt text, commit subjects, or snippets in cache artifacts.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ctxhelm-compiler/src/eval.rs` owns historical eval execution, benchmark suite configs, and report contracts.
- `crates/ctxhelm/src/main.rs` owns `ctxhelm eval history`, `ctxhelm eval health`, and Markdown report rendering.
- `.ctxhelm/benchmarks/refactoringminer-v23.json` is the first fixed v2.3 large-history suite.
- `docs/benchmarking.md` documents source-free benchmark contracts and privacy boundaries.

### Established Patterns
- JSON report structs use serde `camelCase`.
- Historical eval artifacts must remain local-only and source-free.
- New manifest fields should be defaulted so older suite files remain valid.

### Integration Points
- `ctxhelm eval history --cache --parallelism N`
- `ctxhelm eval history --cache --force --parallelism N`
- Benchmark manifest fields: `cacheEnabled`, `forceRefresh`, `parallelism`

</code_context>

<specifics>
## Specific Ideas

- Store cached historical eval reports under the local ctxhelm home by `repo_id` and `eval_range_id`.
- Return cached reports before inventory loading and git sampling when inputs match.
- Add runtime fields for cache hits/misses, effective parallelism, git sample time, ranking time, pack/compiler time, and slow commits.
- Add a regression test that proves cache hits do not store source text or commit prompt text.

</specifics>

<deferred>
## Deferred Ideas

- Fine-grained candidate-feature export belongs to Phase 52.
- Paired baseline and ablation verdicts belong to Phase 53.
- Learned policy experiments belong to Phase 54.

</deferred>
