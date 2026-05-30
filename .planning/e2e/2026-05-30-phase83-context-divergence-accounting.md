# Phase 83: Context Divergence Accounting

## Goal

Make context-vs-all-file product-proof divergence machine-checkable instead of relying on prose notes.

## Change Summary

- Added source-free product-proof verdict fields:
  - `contextVsAllFileDeltaAt10`
  - `lexicalContextVsAllFileDeltaAt10`
  - `allFileDivergenceExplained`
- Product-proof promotion now blocks unexplained all-file lexical deficits.
- `scripts/check-product-proof.py` now fails when divergence fields are missing or when `lexicalDeltaAt10 < -0.03` without `allFileDivergenceExplained = true`.
- Markdown rendering now prints context delta, context-vs-all-file delta, and all-file divergence explanation status.

## Proof Command

```bash
cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxpack-phase83-context-divergence-proof.json

python3 scripts/check-product-proof.py /tmp/ctxpack-phase83-context-divergence-proof.json
cp /tmp/ctxpack-phase83-context-divergence-proof.json \
  .ctxpack/e2e/phase83-context-divergence-proof.json
```

## Result

`releaseGate.decision = promote`

| Corpus | Status | File Recall@10 | Lexical File Recall@10 | File Delta | Context Recall@10 | Lexical Context Recall@10 | Context Delta | Context vs All-File | Lexical Context vs All-File | Explained |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | `match` | 0.600 | 1.000 | -0.400 | 1.000 | 1.000 | +0.000 | +0.400 | +0.000 | true |
| ctxpack | `beat` | 0.446 | 0.306 | +0.140 | 0.444 | 0.306 | +0.139 | -0.002 | -0.001 | true |
| ReAgent | `beat` | 0.500 | 0.550 | -0.050 | 1.000 | 0.571 | +0.429 | +0.500 | +0.021 | true |
| VeriSchema | `beat` | 0.168 | 0.122 | +0.046 | 0.205 | 0.082 | +0.123 | +0.038 | -0.040 | true |

## Interpretation

RefactoringMiner and ReAgent trail lexical on raw all-file recall, but both are now explicitly explained:

- The non-test context channel is non-regressed.
- Validation coverage is maintained through the validation channel.
- The deficit is caused by mixing validation targets into an all-file score, not by losing source context.

The release checker now makes that distinction enforceable. Future source-context losses cannot pass by adding a prose note; they need `allFileDivergenceExplained = true`, which requires non-regressed context recall and covered validation targets.

## Remaining Follow-Up

- Improve low-information and multi-area task detection.
- Continue parser/precision work for dependency and symbol misses.
