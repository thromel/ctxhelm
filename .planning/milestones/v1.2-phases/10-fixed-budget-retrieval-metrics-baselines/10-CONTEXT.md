# Phase 10: Fixed-Budget Retrieval Metrics & Baselines - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Mode:** Auto-generated infrastructure context

<domain>
## Phase Boundary

Extend the v1.2 benchmark reports so maintainers can compare ctxhelm's hybrid retrieval against lexical-only and no-context baselines under fixed budgets. Add token ROI estimates for brief, standard, and deep packs. This phase should remain source-free and should not introduce embeddings, cloud reranking, storage migrations, regression trend comparison, or product proof prose.

</domain>

<decisions>
## Implementation Decisions

### Additive Public Contracts

The historical eval JSON is already a public compatibility surface. Add Phase 10 fields additively to preserve existing consumers while giving future release gates stable keys.

### No-Context Baseline Means Zero Files

Use a zero-file baseline to represent an agent starting without ctxhelm-provided context. This is the cleanest source-free baseline until editor-open-file anchors are available through an integration-specific trace.

### Token ROI Is Estimated, Not Claimed Runtime Cost

Report useful changed-file targets per 1k estimated context tokens for brief, standard, and deep budget options. The metric should identify when a larger pack adds no additional useful target labels.

### Keep Source-Free Boundaries

Metrics may include paths, roles, hashes, counts, budgets, and booleans. They must not include source snippets, prompt text, commit subjects, or external uploads.

</decisions>

<code_context>
## Existing Code Insights

- `crates/ctxhelm-compiler/src/eval.rs` owns historical eval contracts, ranking metrics, lexical baseline comparison, signal ablations, and benchmark suite execution.
- `crates/ctxhelm/src/main.rs` renders historical and benchmark Markdown reports and exposes JSON through `--format json`.
- `crates/ctxhelm-compiler/src/lib.rs` contains contract-shape and source-free unit tests for historical eval reports.
- `crates/ctxhelm/tests/cli_compat.rs` guards CLI JSON and Markdown compatibility.
- `docs/benchmarking.md` and `README.md` describe the eval contract.

</code_context>

<specifics>
## Specific Ideas

- Add `noContextBaseline` and explicit lift-vs-no-context fields to `rankingComparison`.
- Add `tokenRoi` rows with budget, ranking cutoff, estimated tokens, useful targets, safe targets, useful targets per 1k tokens, recall, marginal useful targets, and larger-pack warning.
- Surface the new fields in historical eval Markdown and suite Markdown.
- Extend source-free unit and CLI compatibility tests.

</specifics>

<deferred>
## Deferred Ideas

- Deeper role-filtered segmentation and failure-family mapping belong to Phase 11.
- Trend comparison and regression thresholds belong to Phase 11.
- Product proof docs and release-gate benchmark smoke belong to Phase 12.
- Editor anchor baselines can be added after integration traces capture open-file context.

</deferred>
