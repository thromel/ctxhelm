# Phase 215: Release-Gated Memory Reuse Proof

Date: 2026-06-03

## Goal

Move experience-memory reuse from an optional diagnostic smoke into the required
release proof contract. Phase 214 made memory reuse measurable in historical and
product-proof reports; this phase makes the deterministic approval/reuse
workflow a release-gated invariant.

## What Changed

- `scripts/release-gate.sh` now runs `scripts/smoke-memory-reuse.sh` after the
  general memory smoke.
- The generated release proof bundle includes `scripts/smoke-memory-reuse.sh`
  in `requiredChecks`.
- Release packaging contract tests now require the memory reuse smoke script in
  the release gate.
- Memory, release, and storage docs now describe the release-gated memory reuse
  proof.
- `scripts/check-release-docs.sh` now requires `docs/memory.md` to mention the
  memory reuse smoke.

## Proof Artifact

`.ctxhelm/e2e/phase215-memory-reuse-smoke.json`

Source-free result:

- `status = passed`
- before approval: `targetHit = false`, `selectedMemoryCount = 0`,
  `memorySignalCount = 0`
- after approval: `targetHit = true`, `selectedMemoryCount = 1`,
  `memorySignalCount = 2`
- generated experience cards: `1`
- stored records: `1`
- approved records: `1`
- privacy: local-only, no source text, raw prompts, transcripts, MCP traffic,
  remote embeddings, or remote reranking

## Proof Boundary

This phase proves the deterministic memory approval/reuse workflow and makes it
release-gated. It does not yet prove broad historical memory lift across
RefactoringMiner, ctxhelm, ReAgent, VeriSchema, or other corpora.

## Validation

- `cargo build -p ctxhelm --locked`
- `CTXHELM_BIN="$PWD/target/debug/ctxhelm" bash scripts/smoke-memory-reuse.sh --output .ctxhelm/e2e/phase215-memory-reuse-smoke.json`
- `cargo test -p ctxhelm --test release_packaging --locked`
