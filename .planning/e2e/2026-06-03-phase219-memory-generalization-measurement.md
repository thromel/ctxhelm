# Phase 219 Memory Generalization Measurement

Purpose: move memory R&D from single-pair proof to repeatable real-corpus
measurement. Phase 218 proved parent-snapshot memory visibility; Phase 219
measures whether that visibility generalizes across multiple repeated-file
historical pairs and how much noise memory introduces.

## Change

- Added `scripts/measure-memory-generalization.sh`.
- The script scans a local git repository for repeated-file source-like commit
  pairs.
- For each pair it runs `eval history` on the newer commit before approval,
  seeds one approved source-free experience card from the older task and target
  path, runs `eval history` again, and records aggregate lift/noise/runtime.
- The report stores path labels, commit prefixes, task hashes, counters, and
  booleans only. It does not store raw commit subjects, source snippets,
  terminal logs, model transcripts, or raw MCP traffic.

## Real-Corpus Run

Command:

```bash
cargo build -p ctxhelm --locked
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
  scripts/measure-memory-generalization.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner \
  --pairs 2 \
  --scan-commits 200 \
  --output .ctxhelm/e2e/phase219-refactoringminer-memory-generalization.json
```

Artifact:

- `.ctxhelm/e2e/phase219-refactoringminer-memory-generalization.json`

Result:

- `status = measured`
- `evaluatedPairs = 2`
- `memoryCandidatePairs = 2`
- `memoryTargetHitPairs = 1`
- `memoryUniqueLiftPairs = 1`
- `combinedRecoveredPairs = 1`
- `lexicalCoveredPairs = 1`
- `memoryUniqueTargetHitCount = 1`
- `memoryUniqueNonTargetCount = 8`
- `precisionNeedsWork = true`
- `generalizationProven = false`

## Interpretation

This is a useful R&D result, but not a broad memory win yet.

Memory now reaches real parent-snapshot historical evals and can recover a
target that lexical misses. However, the two-pair RefactoringMiner measurement
also shows high unique non-target memory pressure. The next memory R&D should
focus on selection precision, demotion of low-value memory candidates, and
larger multi-pair/multi-repo measurement before making stronger product claims.

## Validation

- `bash -n scripts/measure-memory-generalization.sh`
- `cargo build -p ctxhelm --locked`
- RefactoringMiner two-pair measurement completed and wrote the source-free
  artifact above.
