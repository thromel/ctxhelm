# Phase 178: Explained All-File Trails

## Goal

Keep product-proof lexical comparison honest after focused context-area and JVM
area guidance exposed a raw all-file RefactoringMiner trail that is covered by
the product's context and validation channels.

## Problem

The clean four-repo proof promoted, but the suite-level all-file lexical claim
still treated every raw target-file trail as an equal product regression. The
RefactoringMiner trail was one changed validation file that ctxhelm recommended
through targeted validation commands, while source/context recall was at lexical
ceiling. That made the headline claim less precise than the per-corpus verdict.

## Change

`releaseGate.lexicalComparison` now keeps the raw counters and adds:

- `allFileExplainedTrailCount`
- `allFileUnexplainedTrailCount`

Explained raw trails remain visible in `allFileTrailCount`, but they count as
match-like for the headline `allFileClaim`. Unexplained trails still force
`allFileClaim = trails_any_corpus`.

## Proof

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase178-broad-proof-home-v2 \
  cargo run -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json \
  --format json > /tmp/ctxhelm-rd/phase178-clean-fixture-explained-trails.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase178-clean-fixture-explained-trails.json
```

Result:

- `releaseGate.decision = promote`
- `allFileClaim = mixed`
- `allFileBeatCount = 3`
- `allFileMatchCount = 0`
- `allFileTrailCount = 1`
- `allFileExplainedTrailCount = 1`
- `allFileUnexplainedTrailCount = 0`
- Average File Recall@10: ctxhelm `0.61190045` vs lexical `0.45709258`
- Average file delta: `+0.15480787`
- Average agent-evidence delta: `+0.2570628`
- Average context delta: `+0.30652046`

Corpus interpretation:

| Corpus | Status | File Recall@10 | Lexical Recall@10 | Source Recall@10 | Agent Evidence Recall@10 | Explained |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | match | 0.8 | 1.0 | 1.0 | 1.0 | true |
| ctxhelm | beat | 0.6666667 | 0.30634922 | 0.46666667 | 0.6666667 | true |
| ReAgent | beat | 0.8 | 0.4 | 1.0 | 0.8 | true |
| VeriSchema | beat | 0.18093514 | 0.122021124 | 0.2763158 | 0.38995475 | true |

## Focused Tests

```bash
cargo test -p ctxhelm-compiler --locked \
  product_proof_release_gate_blocks_mixed_or_trailing_corpora -- --nocapture
cargo test -p ctxhelm-compiler --locked \
  product_proof_release_gate_separates_context_and_validation_channels -- --nocapture
cargo test -p ctxhelm --test cli_compat --locked \
  eval_proof_generates_source_free_product_report -- --nocapture
```

All three focused tests passed.

## Interpretation

This is a proof-accounting fix, not a ranking shortcut. It makes the release
gate match the product contract:

- raw all-file recall remains auditable;
- context-channel and agent-evidence claims remain separate;
- validation-covered raw file trails no longer masquerade as unexplained
  context-selection regressions.
