# Phase 74 Protected Evidence Diagnostics

## Goal

Make protected-evidence pressure actionable by separating exact/symbol candidates
that are actual retrievable targets from protected candidates that are useful
ranking pressure but outside the eval denominator.

## Change

- `HistoricalProtectedEvidenceFile` now records whether the path is a
  `retrievalTarget` and, when known, its source-free file `role`.
- `ProtectedEvidenceSummary` now reports:
  - total protected candidates, misses, and miss-rate;
  - retrieval-target protected candidates, misses, and miss-rate;
  - non-target protected candidates and misses;
  - per-signal retrieval-target candidate/miss counts.
- Product-proof corpus verdicts now include
  `protectedEvidenceTargetMissRateAt10` and notes that show both total and
  retrieval-target protected miss-rate.

This does not change retrieval ranking. It tightens the eval contract so
release evidence can distinguish harmful target misses from non-target
context-pressure misses.

## Required Two-Repo Proof

Command:

```bash
cargo run -p ctxpack -- eval proof \
  --config .ctxpack/e2e/v25-multirepo-baseline-config.json \
  --format json > /tmp/ctxpack-phase74-two-repo-proof.json
python3 scripts/check-product-proof.py /tmp/ctxpack-phase74-two-repo-proof.json
```

Committed artifact:

- `.ctxpack/e2e/phase74-protected-evidence-diagnostics-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected miss@10 | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `beat` | 0.778 | 0.741 | 1.000 | 0.053 | 0.059 |
| ctxpack | `beat` | 0.500 | 0.429 | 1.000 | 0.143 | 0.133 |

Gate decision: `promote`.

Target protected misses now identify the true follow-up files instead of mixing
them with non-target candidates:

- RefactoringMiner: `Constants.java` symbol candidate in commit `1f4d0717c544`.
- ctxpack: `crates/ctxpack-index/src/lib.rs`,
  `crates/ctxpack-compiler/src/ranking.rs`, `.planning/STATE.md`, and
  `docs/benchmarking.md`.

## Broader Fixed-Corpus Probe

Command:

```bash
cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxpack-phase74-broader-proof.json
```

Committed artifact:

- `.ctxpack/e2e/phase74-broader-protected-evidence-diagnostics-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected miss@10 | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `match` | 1.000 | 1.000 | 1.000 | 0.100 | 0.000 |
| ctxpack | `beat` | 0.361 | 0.306 | 0.000 | 0.163 | 0.100 |
| ReAgent | `beat` | 0.714 | 0.571 | 1.000 | 0.217 | 0.000 |
| VeriSchema | `trail` | 0.151 | 0.082 | 0.661 | 0.116 | 0.143 |

Gate decision: `block`.

The broader fixture still blocks promotion. The new diagnostic narrows the
remaining work:

- VeriSchema validation-test Recall@10 remains below the broader 0.80 floor.
- VeriSchema and ctxpack still have protected retrieval-target misses.
- RefactoringMiner and ReAgent show non-target protected pressure, but no
  protected retrieval-target misses on this pinned probe.

## Validation

```bash
cargo test -p ctxpack-compiler protected_evidence -- --nocapture
cargo run -p ctxpack -- eval proof --config .ctxpack/e2e/v25-multirepo-baseline-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxpack-phase74-two-repo-proof.json
cargo run -p ctxpack -- eval proof --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json --format json
```

