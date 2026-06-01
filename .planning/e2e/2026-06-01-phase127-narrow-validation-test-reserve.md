# Phase 127: Narrow Validation-Test Reserve

## Goal

Close the remaining target-file lexical comparison gap caused by narrow tasks whose changed validation tests are correctly recommended through `related_tests` but fall outside the top-10 context-file ranking.

## Change

`context_file_ranking` now reserves validation-test slots only for narrow plans. A plan is treated as narrow when the planner did not emit broad `contextAreas`.

Broad plans keep the existing file-first ranking because tests for broad/multi-area work are already represented through the validation channel and can otherwise crowd out source or governance evidence.

## Rejected Prototype

An unconditional one-third test reserve was rejected.

Evidence:

- RefactoringMiner file recall improved from `0.6` to `1.0`.
- Release gate blocked because ctxpack and VeriSchema protected source evidence regressed.
- VeriSchema protected-evidence target miss rate rose to `0.71428573`.

## Accepted Proof

Command:

```bash
cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json \
  --format json
```

Artifact:

- `.ctxpack/e2e/phase127-narrow-validation-test-reserve.json`

Result:

- `releaseGate.decision = promote`
- `allFileClaim = mixed`
- `allFileBeatCount = 3`
- `allFileMatchCount = 1`
- `allFileTrailCount = 0`
- Average File Recall@10: ctxpack `0.5927659` vs lexical `0.45709258`
- Average file delta: `+0.13567334`
- Agent-evidence delta remains `+0.18792826`
- Context-channel delta remains `+0.23022664`

Corpus results:

| Corpus | File Recall@10 | Lexical Recall@10 | Delta | Protected Target Miss Rate |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | 1.0 | 1.0 | 0.0 | 0.0 |
| ctxpack | 0.3968254 | 0.30634922 | +0.090476185 | 0.083333336 |
| ReAgent | 0.8 | 0.4 | +0.4 | 0.0 |
| VeriSchema | 0.17423832 | 0.122021124 | +0.0522172 | 0.2857143 |

## Validation

Focused tests:

```bash
cargo test -p ctxpack-compiler context_ranking -- --nocapture
```

Result:

- `context_ranking_keeps_validation_tests_inside_budget` passed.
- `context_ranking_keeps_broad_plans_file_first` passed.

## Interpretation

The release proof now shows ctxpack is not trailing lexical search in any measured corpus for raw top-10 target-file recall, while preserving the stronger agent-evidence and context-channel advantages from Phase 126.
