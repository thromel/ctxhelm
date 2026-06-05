# Phase 223 E2E: Memory Corroboration Policy

Date: 2026-06-05

## Goal

Tighten experience-memory precision without hiding honest memory measurements.
The target is to separate true unsupported memory noise from memory attached to
other current retrieval signals.

## Changes

- Ranking no longer attaches memory to lexical-expansion-only paths.
- Ranking only allows one uncorroborated memory-only rescue when no target file
  was otherwise selected.
- Historical memory summaries now report:
  - `memoryUniqueTargetHitWithCurrentSupportCount`
  - `memoryUniqueTargetHitWithoutCurrentSupportCount`
  - `memoryUniqueNonTargetWithCurrentSupportCount`
  - `memoryUniqueNonTargetWithoutCurrentSupportCount`
- The memory generalization harnesses aggregate unsupported-memory precision
  fields and expose `unsupportedMemoryPrecisionNeedsWork`.

## Command

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
  scripts/measure-memory-generalization-suite.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm \
  --pairs 1 \
  --scan-commits 120 \
  --semantic \
  --semantic-provider local_hash \
  --output .ctxhelm/e2e/phase223-memory-corroboration-policy-suite.json
```

## Artifact

- `.ctxhelm/e2e/phase223-memory-corroboration-policy-suite.json`

## Result

The four-repo semantic-enabled run reports:

- `memoryUniqueLiftPairs = 1`
- `memoryUniqueTargetHitCount = 1`
- `memoryUniqueNonTargetCount = 1`
- `memoryUniqueTargetHitWithoutCurrentSupportCount = 0`
- `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`
- `semanticSelectedTargetPairs = 2`
- `unsupportedMemoryPrecisionNeedsWork = false`

## Interpretation

Phase 222's raw `memoryUniqueNonTargetCount = 1` was too coarse. The remaining
memory non-target is supported by another current selected signal, so it is not
pure memory-only noise. The stricter unsupported-memory view is clean on this
four-pair probe, but the sample is still too small for promotion. The next R&D
bar is larger repeated-history pair counts and real-agent outcome lift.
