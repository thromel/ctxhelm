# Phase 216: Historical Memory Lift Proof

Date: 2026-06-03

## Goal

Raise memory reuse proof from deterministic `prepare-task` reuse to historical
eval lift. Phase 215 proved approved memory can promote a target in a live plan;
this phase proves `eval history` can report that memory made a target visible
when lexical still missed it.

## What Changed

- Added `scripts/smoke-memory-history-lift.sh`.
- Added the script to `scripts/release-gate.sh`.
- Added the script to release-gate required checks.
- Added release-packaging contract coverage for the script.
- Updated memory, storage, release, and benchmarking docs.
- Updated release-doc consistency checks.

## Proof Artifact

`.ctxhelm/e2e/phase216-memory-history-lift.json`

Source-free result:

- `status = passed`
- before approval:
  - `evaluatedCommits = 1`
  - `memoryCandidateCount = 0`
  - `memoryUniqueTargetHitCount = 0`
  - `targetInCombined = false`
  - `targetInLexical = false`
  - action includes `collect_or_approve_experience_memory`
- after approval:
  - `evaluatedCommits = 1`
  - `memoryCandidateCount = 1`
  - `memorySelectedAt10Count = 1`
  - `memoryTargetHitAt10Count = 1`
  - `memoryUniqueTargetHitCount = 1`
  - `targetInCombined = true`
  - `targetInLexical = false`
  - action includes `evaluate_memory_reuse_lift`
- privacy: local-only, no source text, raw prompts, transcripts, MCP traffic,
  remote embeddings, or remote reranking

## Proof Boundary

This is a controlled one-commit historical eval designed to prove the memory
lift reporting path. It does not claim broad historical lift across
RefactoringMiner, ctxhelm, ReAgent, VeriSchema, or arbitrary user repos.

## Validation

- `CTXHELM_BIN="$PWD/target/debug/ctxhelm" bash scripts/smoke-memory-history-lift.sh --output .ctxhelm/e2e/phase216-memory-history-lift.json`
