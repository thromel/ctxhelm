# Phase 80 Unique Symbol Floor Accounting

## Goal

Close the remaining ctxpack protected source-symbol target misses without
relaxing product-proof gates or increasing the default context budget.

Phase 79 added source-symbol floors, but proof inspection showed the floor was
still counting duplicate already-selected symbol files against the symbol
budget. That left one symbol-only target below the 10-file budget in both the
required and broader ctxpack slices.

## Change

- Run a bounded source-symbol floor before governance/doc fill.
- Count only unique newly selected files against source-symbol and general
  symbol floor limits.
- Keep archive artifact deferral and source/config/governance floors from Phase
  79 unchanged.

This keeps high-scoring duplicate symbol evidence from consuming the reserve
for symbol-only source files.

## Required Two-Repo Proof

Command:

```bash
cargo run -p ctxpack -- eval proof \
  --config .ctxpack/e2e/v25-multirepo-baseline-config.json \
  --format json > /tmp/ctxpack-phase80-required-proof.json
python3 scripts/check-product-proof.py /tmp/ctxpack-phase80-required-proof.json
```

Committed artifact:

- `.ctxpack/e2e/phase80-unique-symbol-floor-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Effective validation recall | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `beat` | 0.778 | 0.741 | 1.000 | 0.000 |
| ctxpack | `beat` | 0.589 | 0.397 | 0.000 | 0.000 |

Gate decision: `promote`.

The previous required ctxpack miss,
`crates/ctxpack-compiler/src/planning.rs`, is now selected inside the standard
budget.

## Broader Fixed-Corpus Proof

Command:

```bash
cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxpack-phase80-broader-proof.json
python3 scripts/check-product-proof.py /tmp/ctxpack-phase80-broader-proof.json
```

Committed artifact:

- `.ctxpack/e2e/phase80-broader-unique-symbol-floor-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Effective validation recall | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `match` | 1.000 | 1.000 | 1.000 | 0.000 |
| ctxpack | `beat` | 0.444 | 0.306 | 0.000 | 0.000 |
| ReAgent | `beat` | 1.000 | 0.571 | 1.000 | 0.000 |
| VeriSchema | `beat` | 0.205 | 0.082 | 1.000 | 0.000 |

Gate decision: `promote`.

The previous broader ctxpack miss,
`crates/ctxpack-index/src/related_tests.rs`, is now selected inside the standard
budget.

## Validation

```bash
cargo fmt --check
cargo test -p ctxpack-compiler selection_counts_unique_source_symbol_floor_additions -- --nocapture
cargo test -p ctxpack-compiler selection_reserves_symbol_floor -- --nocapture
cargo run -p ctxpack -- eval proof --config .ctxpack/e2e/v25-multirepo-baseline-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxpack-phase80-required-proof.json
cargo run -p ctxpack -- eval proof --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxpack-phase80-broader-proof.json
```
