# Phase 84: Broad Scope Dependency Floors

## Goal

Reduce measured parser/precision dependency misses for broad multi-area tasks without regressing ordinary lexical/doc-heavy tasks.

## Change Summary

- Added `multi_area_task` prepare-task diagnostics for broad workflow/eval/lint prompts.
- Added source-free historical eval fields:
  - `broadScopeCommitCount`
  - per-commit `broadScopeTask`
- Added a bounded source dependency floor in ranking selection.
- Scoped that dependency floor to broad-scope prompts only.

## Regression Found During Development

An unconditional dependency source floor improved VeriSchema but regressed RefactoringMiner:

- RefactoringMiner `fileRecallAt10`: `0.600 -> 0.400`
- RefactoringMiner context verdict: `match -> trail`
- Product proof blocked.

The fix was to activate the dependency floor only for broad-scope prompts. That preserves lexical/doc targets for ordinary tasks while still improving broad workflow/eval tasks.

## Proof Command

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxhelm-phase84-dependency-broad-scope-proof.json

python3 scripts/check-product-proof.py /tmp/ctxhelm-phase84-dependency-broad-scope-proof.json
cp /tmp/ctxhelm-phase84-dependency-broad-scope-proof.json \
  .ctxhelm/e2e/phase84-broad-scope-dependency-proof.json
```

## Result

`releaseGate.decision = promote`

| Corpus | File Recall@10 Before | File Recall@10 After | Source Recall@10 Before | Source Recall@10 After | Broad-Scope Commits |
| --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 0.600 | 0.600 | 1.000 | 1.000 | 0 |
| ctxhelm | 0.446 | 0.446 | 0.633 | 0.633 | 0 |
| ReAgent | 0.500 | 0.500 | 1.000 | 1.000 | 0 |
| VeriSchema | 0.168 | 0.179 | 0.249 | 0.304 | 4 |

## Interpretation

The measurable improvement is narrow and useful:

- VeriSchema broad workflow/eval tasks now preserve more dependency source evidence.
- RefactoringMiner, ctxhelm, and ReAgent remain stable.
- The proof still reports remaining `no_candidate_signal` families, so this does not hide parser/precision work that still needs a deeper fix.

## Remaining Follow-Up

- Address remaining `no_candidate_signal` source families in VeriSchema.
- Improve test-mapping misses for RefactoringMiner, ReAgent, and VeriSchema.
