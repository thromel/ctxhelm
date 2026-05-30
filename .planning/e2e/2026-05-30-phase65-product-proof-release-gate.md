# Phase 65 E2E: v2.5 Product Proof And Release Gate

Date: 2026-05-30

## Objective

Finalize a source-free v2.5 product proof and release gate that can honestly
promote, hold, or block the current retrieval default from measured evidence.

## Implementation

- Added `releaseGate` to `ProductProofReport`.
- Added per-corpus verdicts with retrieval variant, beat/match/trail status,
  Recall@10, lexical Recall@10, lexical delta, Test Recall@10, protected
  evidence miss-rate, runtime, and notes.
- Rendered the release decision in Markdown proof output.
- Extracted product-proof release validation into
  `scripts/check-product-proof.py`.
- Wired `scripts/release-gate.sh` to fail configured benchmark proof when
  `releaseGate.decision != "promote"`.
- Updated docs to state current retrieval mode recommendations and the v2.5
  proof boundary.

## Commands

```bash
cargo test -p ctxpack-compiler product_proof_release_gate -- --nocapture
cargo test -p ctxpack --test release_packaging -- --nocapture
cargo run -p ctxpack -- \
  eval proof --config .ctxpack/e2e/phase62-default-config.json --format json
python3 scripts/check-product-proof.py .ctxpack/e2e/phase65-product-proof.json
bash scripts/check-release-docs.sh
cargo test --workspace --no-fail-fast
cargo run -p ctxpack -- --help
git diff --check
```

`scripts/check-product-proof.py` is expected to reject the current v2.5 proof
because the decision is `block`.

## Product-Proof Result

`ctxpack eval proof --config .ctxpack/e2e/phase62-default-config.json --format json`
produced:

```json
{
  "decision": "block",
  "defaultPromotionAllowed": false,
  "reason": "Blocked because default promotion requires every corpus to beat lexical; failing corpora: RefactoringMiner:Trail, ctxpack:Match."
}
```

Corpus verdicts:

| Corpus | Variant | Status | ctxpack Recall@10 | Lexical Recall@10 | Delta | Test Recall@10 | Protected miss-rate@10 |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `ctxpack_default` | `trail` | 0.7392 | 0.7792 | -0.0400 | 0.0000 | 0.0526 |
| ctxpack | `ctxpack_default` | `match` | 0.1974 | 0.2021 | -0.0046 | 0.0000 | 0.2000 |

## Decision

v2.5 production proof is complete and honest: the current default is useful as
an agent-native context broker, but it is not promoted as lexical-beating
retrieval. The release gate blocks default promotion for the configured
two-repo proof.

## Follow-Up

- Improve test mapping; both corpora still show `testRecallAt10 = 0.0`.
- Address RefactoringMiner lexical trailing status and ctxpack lexical parity.
- Resolve remaining protected symbol budget pressure.
- Future release proof should promote only after every required corpus reaches
  `beat` status with acceptable runtime and privacy.
