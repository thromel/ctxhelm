# Phase 78 Ceiling-Aware Broader Gate

## Goal

Do not block broader production-readiness proof on an impossible-to-beat lexical
ceiling. If lexical retrieval already has perfect non-test context recall, then
ctxpack can only match that corpus. The gate should accept that match only when
ctxpack also has perfect context recall, healthy validation coverage, and zero
protected retrieval-target misses.

## Change

- Added a ceiling-aware promotion path for `match` corpus verdicts.
- Kept ordinary non-ceiling matches as release blockers.
- Kept hard runtime blocking, but treated a single-commit cold historical
  snapshot under 10 seconds as a diagnostic when the corpus is also a perfect
  ceiling match.
- Updated the product-proof checker to accept either `beat` verdicts or safe
  perfect-ceiling `match` verdicts.

This keeps the proof strict for real retrieval parity while avoiding a false
block on saturated fixed-corpus probes.

## Broader Fixed-Corpus Proof

Command:

```bash
cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxpack-phase78-broader-proof.json
python3 scripts/check-product-proof.py /tmp/ctxpack-phase78-broader-proof.json
```

Committed artifact:

- `.ctxpack/e2e/phase78-ceiling-aware-broader-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Validation command recall | Effective validation recall | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `match` | 1.000 | 1.000 | 1.000 | 0.000 | 1.000 | 0.000 |
| ctxpack | `beat` | 0.361 | 0.306 | 0.000 | 0.000 | 0.000 | 0.100 |
| ReAgent | `beat` | 0.714 | 0.571 | 1.000 | 1.000 | 1.000 | 0.000 |
| VeriSchema | `beat` | 0.151 | 0.082 | 0.709 | 1.000 | 1.000 | 0.143 |

Gate decision: `promote`.

RefactoringMiner is accepted as a safe ceiling match because both ctxpack and
lexical have perfect non-test context recall, validation is covered, and there
are zero protected retrieval-target misses. The broader proof still reports
protected target miss diagnostics for ctxpack and VeriSchema; those remain the
next quality-improvement target rather than a promotion blocker.

## Validation

```bash
cargo fmt --check
cargo test -p ctxpack-compiler product_proof_release_gate_accepts_perfect_ceiling_match -- --nocapture
cargo test -p ctxpack-compiler product_proof_release_gate_blocks_hard_cold_runtime_ceiling -- --nocapture
cargo run -p ctxpack -- eval proof --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxpack-phase78-broader-proof.json
```
