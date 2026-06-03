# Phase 218: Parent-Snapshot Memory Lift

Date: 2026-06-03

## Goal

Fix the real historical-eval memory gap exposed by RefactoringMiner: approved
source-repo memory was not visible when `eval history` evaluated a non-root
commit through a parent snapshot, because that snapshot has a different local
storage identity from the source checkout.

## What Changed

- Historical eval now projects approved source-free memory cards from the
  source repo into the parent-snapshot eval root before building the context
  plan.
- Added a unit regression covering parent-snapshot memory reuse.
- Added `scripts/smoke-memory-parent-snapshot-lift.sh`.
- Added the smoke to `scripts/release-gate.sh`.
- Added release-packaging contract coverage for the smoke.
- Updated memory, storage, release, and benchmarking docs plus release-doc
  consistency checks.

## Proof Artifacts

`.ctxhelm/e2e/phase218-memory-parent-snapshot-lift.json`

Controlled release-smoke result:

- `status = passed`
- before approval:
  - `memoryCandidateCount = 0`
  - `memoryUniqueTargetHitCount = 0`
  - `targetInCombined = false`
  - `targetInLexical = false`
- after approval:
  - `memoryCandidateCount = 1`
  - `memorySelectedAt10Count = 1`
  - `memoryTargetHitAt10Count = 1`
  - `memoryUniqueTargetHitCount = 1`
  - `targetInCombined = true`
  - `targetInLexical = false`
  - action includes `evaluate_memory_reuse_lift`
- privacy: local-only, no source text, raw prompts, raw task text,
  transcripts, MCP traffic, remote embeddings, or remote reranking

`.ctxhelm/e2e/phase218-refactoringminer-parent-snapshot-memory.json`

Real-corpus RefactoringMiner pair result:

- `status = passed`
- target: `src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java`
- before approval:
  - memory candidates absent
  - target already present in combined context through non-memory signals
  - target absent from lexical baseline
- after approval:
  - `memoryCandidateCount = 6`
  - `memorySelectedAt10Count = 6`
  - `memoryTargetHitAt10Count = 2`
  - `memoryUniqueTargetHitCount = 1`
  - `targetInCombined = true`
  - `targetInLexical = false`
  - action includes `evaluate_memory_reuse_lift`

## Proof Boundary

The controlled smoke proves parent-snapshot memory visibility and target lift.
The RefactoringMiner artifact proves the same memory visibility issue is fixed
on a real repeated-file history pair, but it does not prove broad memory
generalization across arbitrary histories.

## Validation

- `cargo test -p ctxhelm-compiler historical_eval_projects_source_memory_into_parent_snapshots --locked`
- `cargo build -p ctxhelm --locked`
- `CTXHELM_BIN="$PWD/target/debug/ctxhelm" bash scripts/smoke-memory-parent-snapshot-lift.sh --output .ctxhelm/e2e/phase218-memory-parent-snapshot-lift.json`
- RefactoringMiner single-pair parent-snapshot memory measurement
