# Phase 224 - Expanded Memory Generalization Measurement

Date: 2026-06-05

## Goal

Increase the real-corpus memory-generalization measurement breadth after Phase
223 tightened memory corroboration. The previous semantic-enabled suite only
measured one repeated-file pair per repository, which was too small to judge
whether the stricter policy preserved broad memory lift.

## Changes

- `scripts/measure-memory-generalization.sh` now discovers all repeated-file
  candidates in the scan window, then prefers distinct target files before
  filling duplicate-path repeated pairs.
- Single-repo reports now include `candidatePairCount`,
  `candidateTargetFileCount`, `evaluatedTargetFileCount`, and
  `pairDiversityMeasured`.
- `scripts/measure-memory-generalization-suite.sh` now defaults to three pairs
  per repo and aggregates `candidatePairCount`, `candidateTargetFileCount`,
  `evaluatedTargetFileCount`, `largerPairCountMeasured`, and
  `pairDiversityMeasured`.
- Release-packaging contract tests require the new fields.
- `docs/memory.md` documents the larger default and diversity counters.

## Proof Command

The first expanded run failed before measuring because `CTXHELM_BIN` was passed
as a relative path and the inner harness runs from each fixture repository. The
accepted run used the absolute debug binary path:

```bash
cargo build -p ctxhelm --locked

CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  scripts/measure-memory-generalization-suite.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm \
  --pairs 3 \
  --scan-commits 180 \
  --semantic \
  --semantic-provider local_hash \
  --output .ctxhelm/e2e/phase224-expanded-memory-generalization-suite.json
```

## Evidence

Artifact: `.ctxhelm/e2e/phase224-expanded-memory-generalization-suite.json`

- `evaluatedRepositoryCount = 4`
- `requestedPairsPerRepo = 3`
- `candidatePairCount = 971`
- `candidateTargetFileCount = 256`
- `evaluatedPairs = 12`
- `evaluatedTargetFileCount = 12`
- `largerPairCountMeasured = true`
- `pairDiversityMeasured = true`
- `memoryUniqueLiftPairs = 2`
- `memoryUniqueTargetHitCount = 2`
- `memoryUniqueNonTargetCount = 3`
- `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`
- `memoryUniqueTargetHitWithoutCurrentSupportCount = 0`
- `semanticSelectedTargetPairs = 4`
- `semanticAblationLiftPairs = 0`
- `unsupportedMemoryPrecisionNeedsWork = false`

## Interpretation

The stricter Phase 223 memory policy still produces broader memory lift when the
suite evaluates more than one pair per repo. The remaining raw
`memoryUniqueNonTargetCount = 3` is lexical-baseline-relative noise, but none of
it is unsupported pure-memory noise under the current-signal accounting. The
next R&D step should therefore focus on real-agent outcome lift and, separately,
whether raw lexical-baseline-relative memory non-targets need additional
ranking pressure once paired-agent evidence shows a consumption problem.

## Validation

- `bash -n scripts/measure-memory-generalization.sh scripts/measure-memory-generalization-suite.sh`
- `cargo test -p ctxhelm --test release_packaging memory_generalization --locked`
