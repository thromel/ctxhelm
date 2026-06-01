# Phase 61 Patterns

## Reuse Existing Eval Contracts

Prefer extending `eval benchmark`, `eval baselines`, and product proof contracts over adding a separate multi-repo evaluator. The product already has source-free historical reports, paired baseline rows, gap summaries, token ROI, and privacy status.

## Keep Large Evidence Local

Store full JSON outputs under ignored `.ctxhelm/e2e/`. Commit only concise source-free summaries under `.planning/e2e/`.

## Use Real Repos, Not Toy Fixtures

Unit tests can use small fixtures, but acceptance requires RefactoringMiner plus one other real local repo with enough history.

## Preserve Honest Claims

If ctxhelm trails lexical on a corpus, report that plainly. The milestone goal is quality improvement, not positive-looking wording.
