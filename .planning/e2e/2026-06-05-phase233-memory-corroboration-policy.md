# Phase 233 - Memory Corroboration Policy

Date: 2026-06-05

## Goal

Fix the unsupported memory non-target found by Phase 232 without losing the
measured memory-unique target lift.

## Root Cause

The Phase 232 six-repo run found one unsupported memory non-target in express.
Pair-level reproduction showed the failing shape:

- lexical baseline already covered the repeated target test file
- native semantic/related-test evidence selected test files
- an uncorroborated memory-only source file was still rescued into the context
  pack because target-file selection had no source target

That is too permissive. Memory-only rescue is still useful when there is no
native evidence, but it should not add an unsupported source file when the
agent already has native validation/test evidence to inspect.

## Changes

- `select_target_files` now allows uncorroborated memory-only rescue only when
  no native related-test evidence is available.
- Added a focused ranking regression test:
  `selection_skips_uncorroborated_memory_rescue_when_native_tests_exist`.
- Preserved the existing rescue behavior when memory is the only available
  evidence.

## Validation

Focused tests:

```bash
cargo test -p ctxhelm-compiler selection_ --locked
```

Express before/after:

- before Phase 233:
  - `memoryUniqueNonTargetWithoutCurrentSupportCount = 1`
  - `unsupportedMemoryPrecisionNeedsWork = true`
- after Phase 233:
  - `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`
  - `unsupportedMemoryPrecisionNeedsWork = false`

Six-repo suite after Phase 233:

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
    --output .ctxhelm/e2e/phase233-memory-corroboration-suite.json
```

## Result

- `evaluatedRepositoryCount = 6`
- `evaluatedPairs = 30`
- `memoryUniqueLiftPairs = 2`
- `memoryUniqueTargetHitCount = 2`
- `memoryUniqueNonTargetCount = 1`
- `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`
- `unsupportedMemoryNoiseRepositoryCount = 0`
- `strongSupportedMemoryNoiseRepositoryCount = 1`
- `repositoryDiversityTargetMet = true`

## Interpretation

The uncorroborated memory precision issue is cleared on the six-repo suite
without losing the two measured memory-unique target hits. Remaining memory
noise is supported by current co-change/dependency evidence in express, so the
next local memory R&D returns to strong-signal overlap inspection and
signal-role comparison. Real-agent outcome lift remains separate.
