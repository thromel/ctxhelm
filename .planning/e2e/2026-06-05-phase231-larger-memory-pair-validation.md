# Phase 231 - Larger Memory Pair Validation

Date: 2026-06-05

## Goal

Validate Phase 230's tuned memory policy at a larger same-corpus pair count and
make the harness route away from `increase_pairs_per_repo` once that larger
validation target is met.

## Changes

- Added `largerPairValidationTargetMet` to single-repo and suite
  memory-generalization interpretation.
- Single-repo reports now route to `expand_repository_diversity` after a
  five-pair, five-distinct-target validation target is met.
- Suite reports now route to `expand_repository_diversity` after each evaluated
  repo reaches five measured pairs with distinct targets.

## Validation

Focused checks:

```bash
bash -n scripts/measure-memory-generalization.sh
bash -n scripts/measure-memory-generalization-suite.sh
cargo test -p ctxhelm --test release_packaging memory_generalization_measurement_script_contract --locked
cargo test -p ctxhelm --test release_packaging memory_generalization_suite_measurement_script_contract --locked
```

Larger four-repo suite:

```bash
cargo build -p ctxhelm --locked
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/measure-memory-generalization-suite.sh \
    --repo /Users/romel/Documents/GitHub/RefactoringMiner \
    --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
    --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent \
    --repo /Users/romel/Documents/GitHub/ctxhelm \
    --pairs 5 \
    --scan-commits 260 \
    --semantic \
    --semantic-provider local_hash \
    --output .ctxhelm/e2e/phase231-memory-larger-pair-validation-suite.json
```

## Result

- `status = measured`
- `evaluatedRepositoryCount = 4`
- `evaluatedPairs = 20`
- `evaluatedTargetFileCount = 20`
- `largerPairValidationTargetMet = true`
- `memoryUniqueLiftPairs = 2`
- `memoryUniqueTargetHitCount = 2`
- `memoryUniqueNonTargetCount = 2`
- `memoryUniqueNonTargetWithCurrentSupportCount = 2`
- `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`
- `weakSupportedMemoryNoiseNeedsTuning = false`
- `recommendedNextRAndD` starts with `expand_repository_diversity`

Per-repo memory non-target pressure:

- RefactoringMiner: `0`
- VeriSchema: `2`
- ReAgent: `0`
- ctxhelm: `0`

## Interpretation

The tuned memory policy holds on the larger same-corpus validation slice:
memory-unique target lift is preserved, unsupported memory noise remains zero,
and weak-supported memory noise stays cleared. More same-corpus pair expansion
is no longer the next local bottleneck; the next local memory R&D is repository
diversity and inspection of the remaining VeriSchema strong-signal overlap.
Real-agent outcome lift still requires a non-rate-limited client run.
