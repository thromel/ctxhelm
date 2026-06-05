# Phase 221 Multi-Repo Memory Generalization Suite

Purpose: move memory R&D from single-repo measurement to a bounded multi-repo
generalization proof while preserving the source-free reporting boundary.

## Change

- Added `scripts/measure-memory-generalization-suite.sh`.
- The suite wrapper accepts multiple `--repo` values.
- For each repo it runs `scripts/measure-memory-generalization.sh`.
- It aggregates source-free repo labels, commit prefixes, path labels, hashes,
  counters, booleans, and runtime.
- It does not store raw repo paths, raw commit subjects, source snippets,
  terminal logs, model transcripts, or raw MCP traffic.

## Real-Corpus Suite Run

Command:

```bash
cargo build -p ctxhelm --locked
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
  scripts/measure-memory-generalization-suite.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm \
  --pairs 1 \
  --scan-commits 120 \
  --output .ctxhelm/e2e/phase221-memory-generalization-suite.json
```

Artifact:

- `.ctxhelm/e2e/phase221-memory-generalization-suite.json`

## Result

Suite:

- `requestedRepositoryCount = 4`
- `evaluatedRepositoryCount = 4`
- `errorRepositoryCount = 0`
- `evaluatedPairs = 4`

Aggregate:

- `memoryCandidatePairs = 4`
- `memoryTargetHitPairs = 4`
- `memoryUniqueLiftPairs = 2`
- `memoryUniqueTargetHitCount = 2`
- `combinedRecoveredPairs = 1`
- `lexicalCoveredPairs = 3`
- `memoryUniqueNonTargetCount = 3`
- `memoryUniqueNonTargetPerUniqueTarget = 1.5`

Interpretation:

- `multiRepoMeasured = true`
- `generalizationProven = true`
- `precisionNeedsWork = true`
- `lexicalStillStrong = true`

## Repository Breakdown

- RefactoringMiner: one unique memory lift, one combined-target recovery, two
  unique non-targets.
- VeriSchema: one unique memory lift, no unique non-targets.
- ReAgent: target hit with no unique lift beyond lexical, no unique non-targets.
- ctxhelm: target hit with no unique lift beyond lexical, one unique non-target.

## What This Proves

Memory lift is no longer only a single-repo anecdote. Under the current bounded
criterion, the suite shows useful memory lift across more than one repository.

## What Remains

The pair count is still small and precision is not fully solved. Next R&D should
increase pairs per repo, compare memory selection against graph/semantic
ablations, and measure whether the improved context actually changes agent
outcomes.

## Validation

- `bash -n scripts/measure-memory-generalization-suite.sh`
- `cargo build -p ctxhelm --locked`
- Four-repo suite run completed and wrote the source-free artifact above.
