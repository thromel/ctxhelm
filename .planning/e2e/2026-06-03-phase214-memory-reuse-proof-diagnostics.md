# Phase 214: Memory Reuse Proof Diagnostics

Date: 2026-06-03

## Goal

Make experience-memory reuse measurable in source-free historical and
product-proof reports. The previous state could store and select memory cards,
but the R&D proof layer did not distinguish useful memory hits, absent memory
evidence, or memory noise.

## What Changed

- Added `MemoryReuseSummary` to historical eval reports.
- Historical reports now count:
  - commits with memory candidates
  - total memory candidates
  - selected memory evidence inside the top-10 context budget
  - memory target hits while memory was active
  - memory target misses while memory was active
  - unique memory target hits absent from lexical baseline evidence
  - unique memory non-targets absent from lexical baseline evidence
  - selected memory role counts
- Historical `recommendedResearchActions` now route memory evidence to:
  - `collect_or_approve_experience_memory`
  - `evaluate_memory_reuse_lift`
  - `reduce_memory_retrieval_noise`
  - `improve_memory_selection_policy`
- Product-proof `recommendedResearchActions` now aggregate embedded
  historical report memory evidence to recommend:
  - `prove_memory_reuse`
  - `evaluate_memory_reuse_lift`
  - `reduce_memory_retrieval_noise`
- Markdown historical eval reports now render the memory reuse summary.

## Proof Boundary

This phase does not claim memory improves end-to-end agent outcomes yet. It
adds the source-free measurement layer required to make that claim later.

## Validation

- `cargo test -p ctxhelm-compiler --lib memory_reuse_summary_counts_unique_memory_target_hits --locked`
- `cargo test -p ctxhelm-compiler --lib historical_eval_report_public_json_shape_is_stable --locked`
- `cargo test -p ctxhelm --bin ctxhelm historical_eval_report_renders_source_free_metrics --locked`
- `cargo test -p ctxhelm --test cli_compat --locked`
- `cargo run -p ctxhelm --locked -- eval history --limit 1 --format json | rg 'memoryReuseSummary|collect_or_approve_experience_memory|evaluate_memory_reuse_lift|memoryCandidateCount|memoryUniqueTargetHitCount'`

Focused validation passed before full release validation. The live historical
eval sample emitted `memoryReuseSummary` with `memoryCandidateCount = 0` and
recommended `collect_or_approve_experience_memory`, which is the correct
source-free route when no active memory candidates contribute.
