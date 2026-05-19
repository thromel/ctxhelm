# Plan: Paired Baseline & Ablation Analysis

## Objective

Add a source-free paired analysis surface that compares default ctxpack ranking
with lexical, no-context, signal-only, feedback-weighted, and ablation variants
using the same historical commit corpus.

## Tasks

1. Extend historical commit eval rows with source-free per-signal ranking
   lists so signal-only diagnostics can be computed without source text.
2. Add `PairedBaselineAnalysisReport` contracts with variant rows, thresholded
   verdicts, lexical delta/status, token ROI, validation coverage, signal
   saturation, retrieval gaps, runtime, and privacy status.
3. Add `ctxpack eval baselines` with Markdown and JSON output.
4. Add focused compiler and CLI tests that verify variants, verdicts, signal
   saturation, token ROI, and source-free JSON.
5. Document the paired baseline report and link it from architecture and
   benchmarking docs.
6. Update requirements, roadmap, state, summary, and verification artifacts.

## Verification

- `cargo fmt --all`
- `CARGO_INCREMENTAL=0 cargo check --workspace`
- `CARGO_INCREMENTAL=0 cargo test -p ctxpack-compiler paired_baseline_analysis_reports_variant_verdicts_without_source_text`
- `CARGO_INCREMENTAL=0 cargo test -p ctxpack --test cli_compat eval_baselines_reports_paired_variants_source_free`
- `CARGO_INCREMENTAL=0 cargo run -p ctxpack -- eval baselines --help`
- `CARGO_INCREMENTAL=0 cargo run -p ctxpack -- --help`
- `CARGO_INCREMENTAL=0 cargo test --workspace`
- `git diff --check`
- `gsd-sdk query roadmap.analyze`

