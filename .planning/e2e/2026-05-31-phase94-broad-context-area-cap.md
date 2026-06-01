# Phase 94: Broad Context-Area Cap

## Goal

Improve broad-task agent guidance without perturbing the target-file or
validation channels that already promote under the product proof gate.

Phase 93 removed the cold RefactoringMiner runtime blocker. The remaining
measured VeriSchema pressure is broad top-10 selection: many changed source
files are inside surfaced implementation areas but cannot all fit into the
initial file budget. Phase 94 therefore improves the source-free
`contextAreas` channel rather than forcing more files into the top-10 list.

## Rejected Experiment

I tested an area-diverse top-10 source selector first. It was rejected because
the broad proof regressed VeriSchema:

- File Recall@10: `0.18449473` -> `0.14914028`
- Source Recall@10: `0.31067252` -> `0.24195907`
- Protected target miss-rate: `0.2857143` -> `0.42857143`

That confirms the top-10 target-file budget is already sensitive. The safer
production move is to improve progressive area guidance outside that budget.

## Implementation

- Increased the source-free broad `contextAreas` cap from `12` to `16`.
- Kept `contextAreas` additive: it does not displace target files, tests,
  commands, snippets, or validation evidence.
- Preserved existing ranking behavior for target files.

## Evidence

Focused tests:

```bash
cargo test -p ctxhelm-compiler context_areas -- --nocapture
cargo test -p ctxhelm-compiler broad_source_area_candidates -- --nocapture
```

Broad proof:

```bash
cargo run --release -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > /tmp/ctxhelm-phase94-context-area-cap-proof.json

python3 scripts/check-product-proof.py \
  .ctxhelm/e2e/phase94-context-area-cap-proof.json
```

Committed proof:

- `.ctxhelm/e2e/phase94-context-area-cap-proof.json`

Result:

- `releaseGate.decision = promote`
- VeriSchema broad context-area recall: `0.71851856`
- Previous VeriSchema broad context-area recall: `0.64708996`
- VeriSchema File Recall@10 remains `0.18449473`
- VeriSchema Source Recall@10 remains `0.31067252`
- VeriSchema Test Recall@10 remains `0.7089947`
- VeriSchema Effective Validation Recall@10 remains `1.0`
- VeriSchema protected target miss-rate remains `0.2857143`
- RefactoringMiner still promotes under the hard cold runtime ceiling at
  `4687ms`

## Notes

This is a context-planning improvement, not a file-recall improvement. It makes
wide tasks more useful to agents by surfacing more implementation areas for
progressive inspection while preserving the stable top-10 target-file behavior.
