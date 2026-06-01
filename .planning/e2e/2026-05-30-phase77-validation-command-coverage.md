# Phase 77 Validation Command Coverage

## Goal

Represent broad multi-area validation tasks honestly. Some tasks cannot fit all
changed validation tests into the top-10 related-test list, but a production
agent can still receive the correct verification strategy through a broader
suite command.

## Change

- Added broad validation-scope detection for smoke/workflow/eval tasks whose
  related tests span multiple test areas.
- Added suite-level fallback commands, such as `pytest`, after targeted test
  commands for those broad tasks.
- Added source-free historical eval metrics:
  - `validationCommandRecall`
  - `effectiveValidationRecallAt10`
  - per-commit `validationCommandHits`
  - per-commit `effectiveValidationHitsAt10`
- Updated the product proof gate to use effective validation recall while still
  reporting raw Test Recall@10 separately.

This keeps target-file ranking separate from validation strategy. Individual
test recall remains visible, and broad commands only improve the validation
channel when they actually cover changed test paths.

## Required Two-Repo Proof

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .ctxhelm/e2e/v25-multirepo-baseline-config.json \
  --format json > /tmp/ctxhelm-phase77-two-repo-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase77-two-repo-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase77-validation-command-coverage-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Effective validation recall | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `beat` | 0.778 | 0.741 | 1.000 | 1.000 | 0.059 |
| ctxhelm | `beat` | 0.423 | 0.352 | 0.000 | 0.000 | 0.083 |

Gate decision: `promote`.

The ctxhelm required slice has no validation-test targets in this refreshed
source-free proof, so its `0.000` validation values are not a gate failure.

## Broader Fixed-Corpus Probe

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxhelm-phase77-broader-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase77-broader-validation-command-coverage-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Validation command recall | Effective validation recall | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `match` | 1.000 | 1.000 | 1.000 | 0.000 | 1.000 | 0.000 |
| ctxhelm | `beat` | 0.361 | 0.306 | 0.000 | 0.000 | 0.000 | 0.100 |
| ReAgent | `beat` | 0.714 | 0.571 | 1.000 | 1.000 | 1.000 | 0.000 |
| VeriSchema | `beat` | 0.151 | 0.082 | 0.709 | 1.000 | 1.000 | 0.143 |

Gate decision: `block`.

Phase 77 resolves the VeriSchema validation blocker by recommending broad
validation commands for multi-area smoke/eval tasks. The broader fixture still
blocks because the pinned newest RefactoringMiner slice is a lexical match,
not a context-channel beat.

## Validation

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler broad_validation_tasks_add_suite_fallback_command -- --nocapture
cargo test -p ctxhelm-compiler product_proof_release_gate_accepts_broad_validation_command_coverage -- --nocapture
cargo test -p ctxhelm-compiler validation_command_coverage_recognizes_broad_pytest -- --nocapture
cargo run -p ctxhelm -- eval proof --config .ctxhelm/e2e/v25-multirepo-baseline-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase77-two-repo-proof.json
cargo run -p ctxhelm -- eval proof --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json --format json
```
