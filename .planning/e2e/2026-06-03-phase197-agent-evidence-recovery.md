# Phase 197: Agent Evidence Recovery Accounting

## Goal

Phase 196 improved validation-area guidance, but the proof still made some
validation-heavy commits look worse than the actual agent evidence. Several
files counted as selected-file missed@10 were already available through
`recommendedTests`; `contextAreaNextReadSummary` only counted progressive
`nextReadPaths`.

## Implementation

- Added `agentEvidenceRecoverableCount` to `ContextAreaNextReadSummary`.
- Counted a missed@10 path as agent-evidence recoverable when it appears in:
  - selected `recommendedContextFiles`
  - selected `recommendedTests`
  - progressive context-area `nextReadPaths`
- Kept existing `nextReadRecoverableCount` unchanged, so progressive-read
  behavior remains separately measurable.
- Rendered the new count in historical-eval markdown.
- Added release-gate checker validation:
  - field must be an integer
  - `agentEvidenceRecoverableCount <= missedFileCountAt10`
  - `nextReadRecoverableCount <= agentEvidenceRecoverableCount`

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-compiler \
  broad_context_area_recall_counts_surfaced_areas \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  historical_eval_report_public_json_shape_is_stable \
  --locked -- --nocapture
cargo test -p ctxhelm \
  historical_eval_report_renders_source_free_metrics \
  --locked -- --nocapture
cargo fmt --all -- --check
```

Result: passed.

Fresh release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase197-agent-evidence-home \
  /tmp/ctxhelm-rd/phase197-agent-evidence-recovery-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase197-agent-evidence-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase197-agent-evidence-recovery-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase197-agent-evidence-recovery-proof.json
```

Result: `releaseGate.decision = promote`.

Selected-file metrics stayed unchanged from Phase 196. The new accounting field
shows how much of the selected-file miss set is still available through agent
evidence:

| Repo | Missed@10 | Next-Read Recoverable | Agent-Evidence Recoverable |
| --- | ---: | ---: | ---: |
| RefactoringMiner | `1` | `0` | `1` |
| ctxhelm | `12` | `11` | `11` |
| ReAgent | `0` | `0` | `0` |
| VeriSchema | `39` | `19` | `29` |

## Why It Matters

This does not claim a ranking improvement. It makes the proof more faithful to
the product: ctxhelm serves more than a top-10 selected-file list. For
validation-heavy broad commits, the agent already receives related tests and
progressive area reads; the new field shows that coverage without weakening the
selected-file recall metric.
