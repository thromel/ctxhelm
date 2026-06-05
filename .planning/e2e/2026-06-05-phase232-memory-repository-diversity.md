# Phase 232 - Memory Repository Diversity

Date: 2026-06-05

## Goal

Move memory-generalization R&D beyond the four-repo fixture slice by adding an
explicit repository-diversity target and measuring memory lift/noise across six
repositories.

## Changes

- Added `repositoryDiversityTarget`, `repositoryDiversityTargetMet`, and
  `repositoryDiversityNeedsExpansion` to the memory-generalization suite report.
- Added per-suite repository counters for memory lift and memory noise:
  `memoryLiftRepositoryCount`, `memoryNonTargetRepositoryCount`,
  `unsupportedMemoryNoiseRepositoryCount`, and
  `strongSupportedMemoryNoiseRepositoryCount`.
- Fixed recommendation ordering so unsupported memory precision work is routed
  before broad expansion or strong-signal inspection.

## Corpus

The six-repo diversity run used:

- VeriSchema
- ReAgent
- ctxhelm
- flask
- fd
- express

RefactoringMiner remains covered by the earlier four-repo/same-corpus memory
evidence, but it exceeded the interactive runtime budget for this specific
diversity harness. The diversity report therefore uses six faster repositories
and records no raw repo paths or source text.

## Validation

Focused checks:

```bash
bash -n scripts/measure-memory-generalization.sh
bash -n scripts/measure-memory-generalization-suite.sh
cargo test -p ctxhelm --test release_packaging memory_generalization --locked
```

Six-repo suite:

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/measure-memory-generalization-suite.sh \
    --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
    --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent \
    --repo /Users/romel/Documents/GitHub/ctxhelm \
    --repo /Users/romel/Documents/GitHub/ctxhelm-rd-corpora/flask \
    --repo /Users/romel/Documents/GitHub/ctxhelm-rd-corpora/fd \
    --repo /Users/romel/Documents/GitHub/ctxhelm-rd-corpora/express \
    --pairs 5 \
    --scan-commits 120 \
    --semantic \
    --semantic-provider local_hash \
    --output .ctxhelm/e2e/phase232-memory-repository-diversity-suite.json
```

## Result

- `status = measured`
- `evaluatedRepositoryCount = 6`
- `repositoryDiversityTarget = 6`
- `repositoryDiversityTargetMet = true`
- `evaluatedPairs = 30`
- `evaluatedTargetFileCount = 30`
- `memoryUniqueLiftPairs = 2`
- `memoryUniqueTargetHitCount = 2`
- `memoryLiftRepositoryCount = 1`
- `memoryUniqueNonTargetCount = 2`
- `memoryUniqueNonTargetWithoutCurrentSupportCount = 1`
- `unsupportedMemoryNoiseRepositoryCount = 1`
- `semanticSelectedTargetPairs = 18`
- `semanticAblationLiftPairs = 0`

Per-repo memory pressure:

- VeriSchema: `memoryUniqueLiftPairs = 2`, `memoryUniqueNonTargetCount = 0`
- ReAgent: `memoryUniqueLiftPairs = 0`, `memoryUniqueNonTargetCount = 0`
- ctxhelm: `memoryUniqueLiftPairs = 0`, `memoryUniqueNonTargetCount = 0`
- flask: `memoryUniqueLiftPairs = 0`, `memoryUniqueNonTargetCount = 0`
- fd: `memoryUniqueLiftPairs = 0`, `memoryUniqueNonTargetCount = 0`
- express: `memoryUniqueLiftPairs = 0`, `memoryUniqueNonTargetCount = 2`,
  `memoryUniqueNonTargetWithoutCurrentSupportCount = 1`

## Interpretation

Repository diversity is no longer the immediate local blocker: the suite hit the
six-repo, five-pair-per-repo target. The broader corpus uncovered a more
important precision issue: `express` introduces one unsupported memory
non-target. The next local memory R&D should demote or re-corroborate
uncorroborated memory candidates, then rerun the six-repo suite. Real-agent
outcome lift remains a separate client-availability/evaluation step.
