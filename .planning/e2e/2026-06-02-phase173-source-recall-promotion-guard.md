# Phase 173: Source Recall Promotion Guard

## Goal

Prevent R&D ranking experiments from being promoted when they improve aggregate File Recall@10 by trading away source-code recall.

This phase was motivated by measured rejected experiments after Phase 172. The proof gate promoted some aggregate improvements even when source recall regressed, so the missing product behavior was not another ranking heuristic. It was a stricter promotion guard.

## Rejected Ranking Experiments

### Wider governance-doc reserve

Artifact:

`/tmp/ctxhelm-rd/phase172-benchmark-doc-floor-proof.json`

Outcome:

- Average File Recall@10 improved from `0.59761477` to `0.6190433`
- ctxhelm Source Recall@10 regressed from `0.46666667` to `0.38333336`
- Rejected because it displaced source evidence with docs

### Broad source-history reserve before governance docs

Artifact:

`/tmp/ctxhelm-rd/phase173-source-history-floor-proof.json`

Outcome:

- Average File Recall@10 regressed from `0.61190045` to `0.5963327`
- ctxhelm File Recall@10 regressed from `0.6666667` to `0.60952383`
- VeriSchema protected target miss-rate regressed from `0.0` to `0.14285715`

### Broad source-history reserve after governance docs

Artifact:

`/tmp/ctxhelm-rd/phase173-source-history-after-governance-proof.json`

Outcome:

- Average File Recall@10 regressed from `0.61190045` to `0.6106184`
- VeriSchema File Recall@10 regressed from `0.18093514` to `0.17580695`
- VeriSchema protected target miss-rate regressed from `0.0` to `0.14285715`

### Workspace binary-entrypoint demotion

Artifact:

`/tmp/ctxhelm-rd/phase173-binary-entrypoint-priority-proof.json`

Outcome:

- Average File Recall@10 regressed from `0.61190045` to `0.6047576`
- ctxhelm Source Recall@10 regressed from `0.46666667` to `0.38333336`
- Average brief token ROI regressed from `1.4375` to `1.375`

### Experiments-root auxiliary demotion

Artifact:

`/tmp/ctxhelm-rd/phase173-auxiliary-experiments-proof.json`

Outcome:

- Average File Recall@10 regressed from `0.61190045` to `0.6085671`
- VeriSchema File Recall@10 regressed from `0.18093514` to `0.16760182`
- VeriSchema protected target miss-rate regressed from `0.0` to `0.2857143`

## Accepted Change

Product-proof corpus verdicts now include additive source-channel comparison fields:

- `sourceRecallAt10`
- `lexicalSourceRecallAt10`
- `sourceDeltaAt10`

The release gate now blocks default promotion when:

```text
sourceDeltaAt10 < -0.03
```

This catches aggregate-only wins where ctxhelm improves docs or validation-adjacent context while losing source files that lexical would have found.

## Proof

Final proof artifact:

`/tmp/ctxhelm-rd/phase173-source-recall-gate-proof.json`

Baseline proof artifact:

`/tmp/ctxhelm-rd/phase172-bounded-benchmark-doc-proof.json`

Config:

`.planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json`

Proof result:

- Decision: `promote`
- Average File Recall@10: `0.61190045`
- Average lexical baseline Recall@10: `0.45709258`
- Average ctxhelm lift@10: `+0.15480788`
- Average Test Recall@10: `0.67989415`
- Average brief token ROI: `1.4375`

Source-channel verdicts:

| Corpus | Source Recall@10 | Lexical Source Recall@10 | Source Delta@10 |
| --- | ---: | ---: | ---: |
| RefactoringMiner | `1.0` | `1.0` | `0.0` |
| ctxhelm | `0.46666667` | `0.38333336` | `+0.08333331` |
| ReAgent | `1.0` | `0.6666667` | `+0.3333333` |
| VeriSchema | `0.2763158` | `0.13157895` | `+0.14473686` |

## Validation

Completed during implementation:

- `cargo test -p ctxhelm-compiler product_proof_release_gate_blocks_source_recall_regression --locked`
- `cargo test -p ctxhelm-compiler product_proof_release_gate --locked`
- `cargo run -p ctxhelm --locked -- eval proof --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json --format json`

Full workspace validation is run as closeout.
