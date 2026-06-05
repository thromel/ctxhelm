# Phase 228 Memory Supported-Noise Routing

## Goal

Make the memory-generalization R&D reports distinguish supported lexical-baseline-relative memory non-targets from unsupported pure-memory noise, so the next action is not an over-broad memory demotion.

## Change

- Added `memoryUniqueNonTargetWithCurrentSupportCount` to single-repo and suite reports.
- Added `supportedMemoryNoiseNeedsReview` to report interpretation.
- Made `recommendedNextRAndD` evidence-conditioned:
  - unsupported memory noise still routes to uncorroborated-memory demotion and corroboration-policy tests;
  - supported memory non-targets route to `inspect_supported_memory_non_target_pressure`;
  - raw memory non-targets route to `compare_memory_noise_against_current_signal_roles`.
- Carried the supported-noise interpretation into per-repo suite summaries.

## Proof

Artifact: `.ctxhelm/e2e/phase228-memory-supported-noise-routing-suite.json`

Command:

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/measure-memory-generalization-suite.sh \
    --repo /Users/romel/Documents/GitHub/RefactoringMiner \
    --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
    --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent \
    --repo /Users/romel/Documents/GitHub/ctxhelm \
    --pairs 3 \
    --scan-commits 180 \
    --semantic \
    --semantic-provider local_hash \
    --output .ctxhelm/e2e/phase228-memory-supported-noise-routing-suite.json
```

Result:

- `status = "measured"`.
- `evaluatedRepositoryCount = 4`.
- `evaluatedPairs = 12`.
- `evaluatedTargetFileCount = 12`.
- `memoryUniqueLiftPairs = 2`.
- `memoryUniqueNonTargetCount = 4`.
- `memoryUniqueNonTargetWithCurrentSupportCount = 4`.
- `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`.
- `supportedMemoryNoiseNeedsReview = true`.
- `unsupportedMemoryPrecisionNeedsWork = false`.
- `recommendedNextRAndD` now includes `inspect_supported_memory_non_target_pressure` and does not include `demote_uncorroborated_memory_candidates`.

## Boundary

This phase improves source-free R&D routing and measurement. It does not claim real-agent outcome lift and does not change ranking behavior.
