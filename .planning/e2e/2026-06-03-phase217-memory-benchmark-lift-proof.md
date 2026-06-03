# Phase 217: Memory Benchmark Lift Proof

Date: 2026-06-03

## Goal

Raise memory reuse proof from single historical-report plumbing to
benchmark/product-proof aggregation. Phase 216 proved one controlled
`eval history` report can show memory-only target lift; this phase proves
`eval proof` preserves that lift across multiple embedded repository reports
and routes product-level R&D to memory reuse evaluation.

## What Changed

- Added `scripts/smoke-memory-benchmark-lift.sh`.
- Added the script to `scripts/release-gate.sh`.
- Added the script to release-gate required checks.
- Added release-packaging contract coverage for the script.
- Updated memory, release, and benchmarking docs.
- Updated release-doc consistency checks.

## Proof Artifact

`.ctxhelm/e2e/phase217-memory-benchmark-lift.json`

Source-free result:

- `status = passed`
- suite:
  - `evaluatedRepositoryCount = 2`
  - `evaluatedCommitCount = 2`
  - product-level action includes `evaluate_memory_reuse_lift`
- per embedded repository report:
  - `memoryCandidateCount = 1`
  - `memorySelectedAt10Count = 1`
  - `memoryTargetHitAt10Count = 1`
  - `memoryUniqueTargetHitCount = 1`
  - `memoryUniqueNonTargetCount = 0`
  - `targetInCombined = true`
  - `targetInLexical = false`
- privacy: local-only, no source text, raw prompts, raw task text,
  transcripts, MCP traffic, remote embeddings, or remote reranking

## Proof Boundary

This is a controlled two-repo benchmark/product-proof smoke. It proves that
memory lift survives product-proof aggregation, but it still does not claim
broad experience-memory generalization across RefactoringMiner, ctxhelm,
ReAgent, VeriSchema, or arbitrary user histories.

## Validation

- `CTXHELM_BIN="$PWD/target/debug/ctxhelm" bash scripts/smoke-memory-benchmark-lift.sh --output .ctxhelm/e2e/phase217-memory-benchmark-lift.json`
