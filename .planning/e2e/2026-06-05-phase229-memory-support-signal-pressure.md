# Phase 229 - Memory Support-Signal Pressure

Date: 2026-06-05

## Goal

Make Phase 228's supported memory-noise review actionable by reporting which
current signal families support memory-unique target hits and memory-unique
non-targets.

## Changes

- Added `memoryUniqueTargetHitCurrentSupportSignalCounts` to historical memory
  summaries and memory-generalization aggregate reports.
- Added `memoryUniqueNonTargetCurrentSupportSignalCounts` to historical memory
  summaries and memory-generalization aggregate reports.
- Added `supportedMemoryNoiseDominantSignals` to single-repo and suite
  interpretation.
- Routed supported current-signal memory noise to
  `tune_memory_weight_against_supported_signal_pressure`.

## Validation

```bash
bash -n scripts/measure-memory-generalization.sh
bash -n scripts/measure-memory-generalization-suite.sh
cargo test -p ctxhelm-compiler memory_reuse_summary_counts_unique_memory_target_hits --locked
cargo test -p ctxhelm --test release_packaging memory_generalization_measurement_script_contract --locked
cargo test -p ctxhelm --test release_packaging memory_generalization_suite_measurement_script_contract --locked
```

Full four-repo suite:

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
    --output .ctxhelm/e2e/phase229-memory-support-signal-pressure-suite.json
```

## Result

- `status = measured`.
- `evaluatedRepositoryCount = 4`.
- `evaluatedPairs = 12`.
- `memoryUniqueLiftPairs = 2`.
- `memoryUniqueNonTargetCount = 4`.
- `memoryUniqueNonTargetWithCurrentSupportCount = 4`.
- `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`.
- `memoryUniqueNonTargetCurrentSupportSignalCounts`:
  - `dependency = 4`
  - `lexical_expansion = 2`
  - `symbol = 2`
  - `semantic = 1`
- `memoryUniqueTargetHitCurrentSupportSignalCounts`:
  - `dependency = 2`
  - `co_change = 1`
  - `semantic = 1`
- `supportedMemoryNoiseDominantSignals = ["dependency", "lexical_expansion", "symbol"]`.

## Interpretation

The remaining memory non-targets are not unsupported pure-memory noise. They are
files current retrieval already supports through dependency, lexical-expansion,
symbol, or semantic signals. The next local memory R&D step should tune memory
weighting against supported signal pressure rather than demoting memory
globally. Real-agent outcome lift remains unproven while Claude Code runs are
rate-limited.
