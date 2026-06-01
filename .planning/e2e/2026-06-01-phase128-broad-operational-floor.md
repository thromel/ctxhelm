# Phase 128: Broad Operational Floors

## Goal

Reduce the remaining broad-task protected target misses without using gold labels
or changing ctxpack's local-first/source-free proof boundary.

## Change

Broad target-file selection now spends bounded budget on operational evidence
before later symbol/dependency expansion:

- root governance docs for planning/proof tasks
- exact config matches
- workflow scripts such as bootstrap, start, run, smoke, check, and test scripts

Deployment/release/publish scripts remain eligible, but they rank behind
setup/run/smoke lifecycle scripts in the workflow-script reserve.

## Proof

Command:

```bash
cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json \
  --format json > /tmp/ctxpack-phase128-proof.json
python3 scripts/check-product-proof.py .ctxpack/e2e/phase128-broad-operational-floor.json
```

Durable artifact:

- `.ctxpack/e2e/phase128-broad-operational-floor.json`

Result:

- `releaseGate.decision = promote`
- `allFileBeatCount = 3`
- `allFileMatchCount = 1`
- `allFileTrailCount = 0`
- average File Recall@10 `0.5986343` vs lexical `0.45709258`
- average file delta `+0.14154172`
- average agent-evidence delta `+0.19379663`
- average context delta `+0.23717105`
- protected target miss-rate `0.0` on RefactoringMiner, ctxpack, ReAgent, and VeriSchema

Compared with Phase 127, the change clears the ctxpack planning-doc protected
miss and the VeriSchema workflow-script protected miss while preserving the
zero-trailing-corpus lexical comparison.

## Focused Tests

```bash
cargo test -p ctxpack-compiler broad_selection_reserves -- --nocapture
```

The focused tests prove that broad selection reserves:

- root governance docs before symbol expansion can consume the full budget
- bootstrap/start/run workflow scripts before deployment scripts when the task
  budget cannot include every operational script

## Notes

This is intentionally a ranking fix, not a new retriever. The signal comes from
existing lexical candidates plus source-free path/role metadata, and it does not
consult `retrievalTargetFiles`, changed files, or protected-evidence labels.
