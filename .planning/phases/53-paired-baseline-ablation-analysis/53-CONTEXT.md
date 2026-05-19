# Phase 53 Context: Paired Baseline & Ablation Analysis

## Goal

Maintainers can compare ctxpack against lexical, no-context, signal-only, and
signal-ablation baselines with honest thresholded verdicts on the same
source-free historical corpus.

## Inputs

- Phase 50 fixed benchmark corpus manifests.
- Phase 51 historical eval cache, deterministic parallelism, and runtime
  diagnostics.
- Phase 52 source-free candidate feature export lifecycle.
- Existing historical eval metrics: combined ranking, lexical baseline,
  no-context baseline, token ROI, retrieval gap taxonomy, runtime, and signal
  ablations.

## Constraints

- Preserve local-first, read-only behavior.
- Do not store source snippets, prompts, commit subjects, terminal logs, stack
  traces, or secret-bearing values.
- Keep paired comparisons on the same commit corpus and K budget.
- Treat lexical parity honestly; do not hide exact-token strength behind
  aggregate ctxpack scores.

## Implementation Areas

- `crates/ctxpack-compiler/src/eval.rs`
- `crates/ctxpack-compiler/src/lib.rs`
- `crates/ctxpack/src/main.rs`
- `crates/ctxpack/tests/cli_compat.rs`
- `docs/benchmarking.md`
- `docs/paired-baselines.md`

