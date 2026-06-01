# Phase 97: Broad Governance Classification

## Goal

Reduce true `no_candidate_signal` misses for governance, proof, and historical
evaluation tasks without changing privacy posture or adding MCP surface area.

Phase 96 made broad areas resource-backed. The next measured gap was that some
historical ctxhelm commits used natural task wording such as "evaluate
retrievable historical targets" or "promote channel aware product proof". Those
tasks were not always classified as governance or broad multi-area work, so
planning docs and source areas could remain missing or under-explained.

## Implementation

- Expanded project-governance task detection to include evaluation verbs and
  proof/promotion/metric language.
- Expanded broad multi-area task detection to include product-proof,
  historical-target, metric, and promotion language.
- Kept candidate generation source-free and local-only.
- Kept MCP tools unchanged.
- Added focused tests for the newly covered governance and broad-task phrases.

## Evidence

Focused tests:

```bash
cargo test -p ctxhelm-compiler governance_tasks_add_root_planning_docs_as_candidates -- --nocapture
cargo test -p ctxhelm-compiler product_proof_and_historical_eval_tasks_are_multi_area -- --nocapture
```

Broad proof:

```bash
cargo run --release -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > .ctxhelm/e2e/phase97-broad-governance-classification-proof.json

python3 scripts/check-product-proof.py \
  .ctxhelm/e2e/phase97-broad-governance-classification-proof.json
```

Committed proof:

- `.ctxhelm/e2e/phase97-broad-governance-classification-proof.json`

Result:

- `releaseGate.decision = promote`
- ctxhelm File Recall@10 improves from `0.44603175` to `0.47460318`
- ctxhelm Source Recall@10 improves from `0.6333333` to `0.7166667`
- ctxhelm broad context-area recall improves from `0.0` to `1.0`
- ctxhelm true source `no_candidate_signal` gaps are converted into
  `area_context_only` or ranked-below-budget candidate families
- VeriSchema File Recall@10 remains `0.18449473`
- VeriSchema Source Recall@10 remains `0.31067252`
- VeriSchema Test Recall@10 remains `0.7089947`
- VeriSchema Effective Validation Recall@10 remains `1.0`
- RefactoringMiner remains under the hard cold runtime ceiling at `3572ms`

## Notes

This phase does not claim to solve broad top-10 budget pressure. It improves
whether broad governance tasks receive the right source-free planning and area
signals so agents can continue with native reads when the top-10 file budget is
too narrow.
