# Phase 66 Summary: Test Recall Evaluation Channel

Date: 2026-05-30

## Result

Completed a production-hardening fix for validation-test evaluation. The product proof now measures Test Recall@K against `recommended_tests`, which is the dedicated validation channel returned by `prepare_task`, instead of treating tests as missing whenever target files fill the flat context-file ranking.

## Changes

- Updated historical eval test-hit calculation to use `recommended_tests`.
- Preserved the existing target-file ranking behavior after proving a blunt test-slot reserve hurt source/file recall.
- Improved related-test ordering by sorting internally on raw score before capped output confidence.
- Added a source-seed priority bonus for exact test-name matches, so tests tied at capped confidence preserve stronger source-seed relationships.
- Added a regression test for exact Java test ordering across RefactoringMiner-style MCP classes.

## Proof

`ctxhelm eval proof --config .ctxhelm/e2e/phase62-default-config.json --format json` wrote `.ctxhelm/e2e/phase66-test-recall-proof.json`.

| Corpus | Status | ctxhelm Recall@10 | Lexical Recall@10 | Delta | Test Recall@10 | Protected miss-rate@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `trail` | 0.7392 | 0.7792 | -0.0400 | 1.0000 | 0.0526 |
| ctxhelm | `match` | 0.1879 | 0.1984 | -0.0105 | 1.0000 | 0.1700 |

## Decision

This fixes the false zero-test-recall signal, but it does not promote the default. The release gate correctly remains `block` because the default still does not beat lexical on every corpus.

## Next Blockers

- Improve RefactoringMiner file Recall@10 beyond lexical baseline.
- Recover ctxhelm source/file recall and address docs/scripts no-candidate gaps.
- Reduce protected evidence miss-rate on ctxhelm.
- Add parser/precision improvements for repeated `ranked_below_budget_dependency` gaps.
