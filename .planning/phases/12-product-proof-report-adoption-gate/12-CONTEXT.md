# Phase 12: Product Proof Report & Adoption Gate - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Mode:** Auto-generated infrastructure context

<domain>
## Phase Boundary

Expose a credible source-free product proof path for v1.2: a maintainer command that runs a configured benchmark suite, summarizes headline metrics, states limitations and where ctxpack helps or does not help, and optionally runs inside the release gate.

</domain>

<decisions>
## Implementation Decisions

### Proof Command Wraps Benchmark Suites

The proof command should reuse the benchmark suite runner instead of introducing another corpus path.

### Release Gate Is Optional

The release gate should only run benchmark proof when `CTXPACK_BENCHMARK_CONFIG` is set. This keeps normal release verification deterministic and does not require local copies of RefactoringMiner or another external repository.

### Honest Product Messaging

The report should state limitations and non-helpful cases. v1.2 should not claim universal agent improvement.

</decisions>

<code_context>
## Existing Code Insights

- `BenchmarkSuiteReport` already contains source-free metrics and privacy status.
- `ctxpack eval benchmark` and `ctxpack eval compare` provide the underlying reproducible command paths.
- `scripts/release-gate.sh` already has optional evidence hooks and selected-binary handling.
- `docs/benchmarking.md`, `README.md`, and `docs/release.md` are the right docs surfaces.

</code_context>

<specifics>
## Specific Ideas

- Add `ctxpack eval proof --config <suite.json>`.
- Add `ProductProofReport` with headline metrics, limitations, helps/does-not-help lists, future work, and embedded benchmark report.
- Add optional `CTXPACK_BENCHMARK_CONFIG` release-gate proof.
- Update docs and future milestone requirements from measured gap taxonomy fields.

</specifics>

<deferred>
## Deferred Ideas

- Hard release-gate thresholds for benchmark regressions can be configured later on top of `ctxpack eval compare`.
- Public static benchmark numbers should be published only after running the configured real-repo suite on the target machine.

</deferred>
