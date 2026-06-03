# Phase 200 - Contextual README Doc Reserve

## Goal

Use Phase 199 candidate-miss pressure to make a measured ranking improvement
without broad top-10 churn. The most actionable recoverable miss was a
docs-only VeriSchema commit where `paper/pvldb/README.md` was generated as a
candidate but lost to adjacent source/docs files.

## Rejected Experiment

Before the accepted change, a source-dependency-before-workflow ordering was
tested:

- Proof: `/tmp/ctxhelm-rd/phase200-source-dependency-before-workflow-proof.json`
- Result: `releaseGate.decision = promote`, but the change regressed metrics.
- Average File Recall@10 moved from `0.658268` to `0.6566013`.
- VeriSchema File Recall@10 moved from `0.35529414` to `0.34862745`.
- Root cause: the ordering displaced a true workflow-script hit in VeriSchema.

That experiment was reverted.

## Accepted Change

Broad task selection now reserves one nested README docs candidate after
governance/config/workflow evidence has first chance. The reserve is source-free
and only applies to docs candidates that already entered retrieval; it does not
read source text or create new candidates.

Focused validation:

```bash
cargo test -p ctxhelm-compiler ranking::tests:: --locked -- --nocapture
```

Result: all ranking tests pass, including
`broad_selection_reserves_contextual_readme_docs`.

## Fresh Proof

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase200-contextual-readme-after-workflow-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase200-contextual-readme-after-workflow-proof.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase200-contextual-readme-after-workflow-proof.json
```

Result: `releaseGate.decision = promote`.

## Deltas Versus Phase 199

- Average File Recall@10: `0.658268 -> 0.7082679`
- Average lexical File Recall@10: unchanged at `0.4795285`
- Average file delta vs lexical: `+0.17873949 -> +0.22873944`
- Average Agent Evidence Recall@10: `0.76052284 -> 0.81052285`
- Average Agent Evidence delta: `+0.28099436 -> +0.33099437`
- Average Context Recall@10: `0.7638889 -> 0.7708334`
- Average Context delta: `+0.32440478 -> +0.33134925`
- All-file beat/match/trail: unchanged at `3 / 0 / 1`
- Explained/unexplained trail: unchanged at `1 / 0`
- Agent-evidence beat/match/trail: unchanged at `3 / 1 / 0`
- Context beat/match/trail: unchanged at `3 / 1 / 0`

Per repository:

- RefactoringMiner: no metric changes.
- ctxhelm: no metric changes.
- ReAgent: no metric changes.
- VeriSchema File Recall@10: `0.35529414 -> 0.55529416`
- VeriSchema Source Recall@10: unchanged at `0.5277778`
- VeriSchema Test Recall@10: unchanged at `0.7896825`
- VeriSchema Effective Validation Recall@10: unchanged at `1.0`
- VeriSchema Broad Context Area Recall: unchanged at `0.84444445`

## Interpretation

The accepted change shows that candidate pressure was not only source/test
pressure. A docs-only nested README target was recoverable through the candidate
set but needed a tiny contextual-doc reserve. The reserve is deliberately placed
after config/workflow evidence to avoid repeating the rejected source-dependency
ordering regression.

Remaining R&D should still target source/test pressure in `schema_agent/agents`,
`tests/agents`, and `tests/evaluation`, but only with rules that prove lift
without displacing known workflow/config hits.
