# Phase 234 - Memory Pack Impact Accounting

Date: 2026-06-05

## Goal

Resolve the remaining Phase 233 strong-signal memory overlap without changing
ranking when the evidence shows memory did not alter the final context pack.

## Root Cause

The Phase 233 express rerun still reported one memory unique non-target:
`lib/application.js`. Pair-level reproduction showed the before-memory and
after-memory top-10 context packs were identical:

- `lib/application.js` was already selected by native dependency/co-change
  evidence.
- Memory also selected two retrieval targets, `lib/response.js` and
  `History.md`.
- Memory changed zero final context files.

The harness was counting signal-level memory overlap as precision work even
when memory had no user-visible pack impact.

## Changes

- `scripts/measure-memory-generalization.sh` now records memory pack-impact
  fields:
  - `memoryPackChangedPairs`
  - `memoryPackTargetGainPairs`
  - `memoryPackAddedFileCount`
  - `memoryPackRemovedFileCount`
  - `memoryPackAddedTargetCount`
  - `memoryPackAddedNonTargetCount`
  - `memorySignalOnlyNonTargetCount`
- `scripts/measure-memory-generalization-suite.sh` aggregates those fields
  across repositories.
- Precision and supported-noise routing now use final-pack non-target additions
  rather than raw signal-only memory overlap.
- Signal-only overlap remains visible through
  `signalOnlyMemoryOverlapObserved` and `track_signal_only_memory_overlap`.
- Release packaging contract tests now require the new fields and routing.

## Validation

Focused tests:

```bash
bash -n scripts/measure-memory-generalization.sh
bash -n scripts/measure-memory-generalization-suite.sh
cargo test -p ctxhelm --test release_packaging memory_generalization --locked
```

Express proof:

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/measure-memory-generalization.sh \
    --repo /Users/romel/Documents/GitHub/ctxhelm-rd-corpora/express \
    --pairs 5 \
    --scan-commits 120 \
    --semantic \
    --semantic-provider local_hash \
    --output .ctxhelm/e2e/phase234-express-pack-impact-memory-overlap.json
```

Six-repo proof:

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
    --output .ctxhelm/e2e/phase234-memory-pack-impact-suite.json
```

## Result

Express:

- `memoryPackChangedPairs = 0`
- `memoryPackAddedNonTargetCount = 0`
- `memorySignalOnlyNonTargetCount = 1`
- `precisionNeedsWork = false`
- `memoryNeedsCorroboration = false`
- `supportedMemoryNoiseNeedsReview = false`

Six-repo suite:

- `evaluatedRepositoryCount = 6`
- `evaluatedPairs = 30`
- `memoryUniqueLiftPairs = 2`
- `memoryUniqueTargetHitCount = 2`
- `memoryUniqueNonTargetCount = 1`
- `memoryPackChangedPairs = 0`
- `memoryPackAddedNonTargetCount = 0`
- `memorySignalOnlyNonTargetCount = 1`
- `unsupportedMemoryNoiseRepositoryCount = 0`
- `precisionNeedsWork = false`
- `generalizationProven = true`

## Interpretation

The remaining express non-target is signal-only overlap, not a context-pack
regression. Memory still has two unique target hits across the six-repo suite,
adds no final-pack non-targets, and clears the local precision-work gate. The
next R&D step is real-agent outcome lift, while signal-only overlap remains a
tracked diagnostic.
