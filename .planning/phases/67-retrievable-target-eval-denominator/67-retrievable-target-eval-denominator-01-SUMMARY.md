# Phase 67 Summary: Retrievable Target Eval Denominator

Date: 2026-05-30

## Result

Completed an eval-correctness hardening pass. Historical eval now distinguishes all safe changed files from files that existed in the parent snapshot and therefore could be retrieved as context.

## Changes

- Added `retrievalTargetFiles` to `HistoricalCommitEval`.
- File Recall@K, lexical Recall@K, MRR, signal-only metrics, token ROI, role recall, missing files, and gap summaries now use `retrievalTargetFiles`.
- `safeChangedFiles` remains the full source-free changed-file audit list.
- Added a regression test for a commit that modifies an existing source file while adding a new docs file.

## Proof

`ctxpack eval proof --config .ctxpack/e2e/phase62-default-config.json --format json`
wrote `.ctxpack/e2e/phase67-retrievable-target-proof.json`.

| Corpus | Status | ctxpack Recall@10 | Lexical Recall@10 | Delta | Test Recall@10 | Protected miss-rate@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `trail` | 0.7392 | 0.7792 | -0.0400 | 1.0000 | 0.0526 |
| ctxpack | `match` | 0.2277 | 0.2326 | -0.0049 | 1.0000 | 0.1500 |

## Decision

The denominator is now more accurate and ctxpack Recall@10 improved on the ctxpack corpus, but the default still does not beat lexical. The release gate correctly remains `block`.

## Next Blockers

- RefactoringMiner source dependency misses still need parser/precision improvements.
- ctxpack still misses repeated existing `.planning`, `docs`, and `scripts` targets.
- ctxpack source files in `crates/ctxpack-compiler/src` and `crates/ctxpack-index/src` are still frequently ranked below budget.
- Protected evidence miss-rate remains non-zero.
