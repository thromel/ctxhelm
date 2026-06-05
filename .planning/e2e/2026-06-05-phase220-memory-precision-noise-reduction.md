# Phase 220 Memory Precision Noise Reduction

Purpose: reduce the real-corpus memory noise exposed by Phase 219 without
losing the one measured unique memory target lift.

## Change

- Experience card generation now preserves eval-trace recommendation order.
- Recommended files stay before recommended tests.
- Duplicate source links are removed without sorting.
- Memory source-link candidate injection is capped to a small source-like
  context set per card.
- Tests are not injected into the ranked context budget through memory source
  links. They remain available through selected memory links and normal
  validation channels.

## Real-Corpus Rerun

Command:

```bash
cargo build -p ctxhelm --locked
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
  scripts/measure-memory-generalization.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner \
  --pairs 2 \
  --scan-commits 200 \
  --output .ctxhelm/e2e/phase220-refactoringminer-memory-precision.json
```

Artifact:

- `.ctxhelm/e2e/phase220-refactoringminer-memory-precision.json`

## Before/After

Phase 219:

- `memoryCandidatePairs = 2`
- `memoryTargetHitPairs = 1`
- `memoryUniqueLiftPairs = 1`
- `combinedRecoveredPairs = 1`
- `memoryUniqueTargetHitCount = 1`
- `memoryUniqueNonTargetCount = 8`

Phase 220:

- `memoryCandidatePairs = 1`
- `memoryTargetHitPairs = 1`
- `memoryUniqueLiftPairs = 1`
- `combinedRecoveredPairs = 1`
- `memoryUniqueTargetHitCount = 1`
- `memoryUniqueNonTargetCount = 2`

## Interpretation

The precision change preserves the measured useful memory lift while removing
six of eight unique non-target memory selections on the current two-pair
RefactoringMiner slice.

This is not broad memory generalization proof. The artifact still reports
`generalizationProven = false` and `precisionNeedsWork = true`. The next R&D
bar is a larger multi-pair and multi-repo memory selection benchmark, plus a
stricter policy for when memory should enter the top context budget.

## Validation

- `cargo fmt --check`
- `cargo test -p ctxhelm-compiler experience_cards_preserve_recommended_file_order_before_tests --locked`
- `cargo test -p ctxhelm-compiler memory_path_candidates_cap_context_links_and_skip_tests --locked`
- RefactoringMiner two-pair measurement completed and wrote the source-free
  artifact above.
