# Phase 230 - Supported Memory-Pressure Tuning

Date: 2026-06-05

## Goal

Turn Phase 229's supported memory-noise pressure profile into a ranking change
that reduces memory-backed non-target pressure without losing memory target
lift.

## Change

Memory source-link evidence now attaches to an existing candidate only when the
candidate already has strong current support:

- `anchor`
- `current_diff`
- `lexical`
- `semantic`
- `co_change`

Dependency-only and symbol-only candidates keep their own ranking evidence, but
memory no longer adds extra pressure on top of those weaker support signals.
Memory-only candidates are still available for the existing one-file rescue path
when no target files were selected at all.

## Validation

Focused ranking tests:

```bash
cargo test -p ctxhelm-compiler ranking_skips_memory_attachment_when_only_weak_current_signals_support_it --locked
cargo test -p ctxhelm-compiler ranking_does_not_attach_memory_to_lexical_expansion_only_path --locked
cargo test -p ctxhelm-compiler selection_keeps_semantic_corroborated_memory_as_supported_source --locked
```

Four-repo memory suite:

```bash
cargo build -p ctxhelm --locked
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
    --output .ctxhelm/e2e/phase230-memory-supported-pressure-tuning-suite.json
```

## Result

Phase 230 suite:

- `status = measured`
- `evaluatedRepositoryCount = 4`
- `evaluatedPairs = 12`
- `memoryUniqueLiftPairs = 2`
- `memoryUniqueTargetHitCount = 2`
- `memoryUniqueNonTargetCount = 1`
- `memoryUniqueNonTargetWithCurrentSupportCount = 1`
- `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`
- `memoryUniqueNonTargetCurrentSupportSignalCounts = { dependency: 1, semantic: 1 }`
- `weakSupportedMemoryNoiseNeedsTuning = false`
- `recommendedNextRAndD` includes
  `inspect_remaining_strong_signal_memory_overlap`.

Compared with Phase 229:

- Memory lift stayed at `2`.
- Unsupported memory non-targets stayed at `0`.
- Raw memory unique non-targets dropped from `4` to `1`.
- Symbol-only and dependency/lexical-expansion supported memory pressure no
  longer survives as memory evidence in the measured suite.

## Interpretation

The local weak-signal pressure tuning succeeded on the measured four-repo
slice. The remaining memory R&D gap is larger pair-count validation, inspection
of the one remaining strong-signal overlap, and real-agent outcome lift, not
another blind memory demotion.
