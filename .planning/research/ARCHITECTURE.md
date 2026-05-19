# Architecture Research: v2.3 Evaluation Lab & Learned Retrieval Policy

## Integration Points

- `crates/ctxpack-core/src/contracts.rs`
- `crates/ctxpack-index/src/git.rs`
- `crates/ctxpack-index/src/storage.rs`
- `crates/ctxpack-compiler/src/eval.rs`
- `crates/ctxpack-compiler/src/benchmark.rs`
- `crates/ctxpack-compiler/src/policy.rs`
- `crates/ctxpack/src/main.rs`
- `scripts/release-gate.sh`
- `docs/benchmarking.md`
- `.planning/e2e/2026-05-19-refactoringminer-full-e2e.md`

## Build Order

1. Define corpus manifests and lock the RefactoringMiner regression suite.
2. Add eval cache/reuse semantics before expanding corpora so iteration stays practical.
3. Export source-free candidate features and paired comparison rows.
4. Add baseline/ablation analysis and thresholded verdicts.
5. Add offline learned-policy experiment and explicit apply/disable gates.
6. Wire bounded eval proof into docs and release gates.

## Architecture Boundary

The learning layer should consume only source-free features:

- path role and extension
- candidate kind
- source signal scores
- rank positions
- graph distances
- history/co-change counts
- test relation confidence
- memory/feedback usage counts
- whether the candidate was eventually read, edited, validated, or part of hidden gold labels

It should not consume raw source, prompts, snippets, terminal logs, issue descriptions, or private repo text.

## Why This Before v2.4

Production embeddings, SCIP automation, and cloud rerankers can all add cost and complexity. v2.3 builds the measurement harness that can later answer whether those backends improve Recall@K, precision, token ROI, runtime, and agent outcomes enough to justify adoption.
