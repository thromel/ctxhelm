# Phase 64 Patterns

## Fix Candidate Absence Before Reranking

If a gold file is marked `no_candidate_signal`, reranking cannot recover it.
First make the file eligible through a narrow retriever or source-free expansion.

## Use Gap Families, Not Anecdotes

A single missed file is not enough. Group misses by role, signal gap,
path-family, target status, and recommendation area, then target repeated
families.

## Keep Tests Separate

Source recall and test recall answer different questions. A change that improves
test recall while losing source targets is not automatically a win.

## Preserve Exact Seeds

Graph and path-family expansion may add candidates around exact seeds. It must
not recurse from weak semantic-only seeds when exact lexical, path, or symbol
seeds exist.

## Measure Collateral Damage

Every targeted fix should be rerun on the two-repo corpus. A RefactoringMiner
gain that harms ctxpack or protected evidence should remain opt-in or be revised.
