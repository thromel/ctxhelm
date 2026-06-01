# Phase 98: Progressive Broad Classification

## Goal

Expose source-free broad context-area guidance for archive/docs retrieval tasks
without spending target-file budget on broad source floors unless the task is
actually implementation/eval shaped.

Phase 97 still had two ctxhelm historical commits that were too broad for a
small file pack but were not recognized as broad context-area tasks:

- `dampen archive artifacts retrieval`
- `MCP docs record real client mcp proof refresh`

A direct classifier expansion made both tasks broad, but the first proof attempt
regressed ctxhelm File Recall@10 from `0.47460318` to `0.34444445` because broad
source floors displaced docs in the 10-file target budget. The accepted fix
separates source-free context-area guidance from target-file source-floor
spending.

## Implementation

- Expanded `is_multi_area_task` to classify archive/artifact/retrieval and
  docs/proof wording as broad for diagnostics and context-area generation.
- Added `uses_broad_target_file_floors` so only implementation/eval-shaped broad
  tasks spend target-file budget on broad source-area candidates and dependency
  floors.
- Kept archive/docs broad tasks source-free and progressive: agents now see
  context areas and MCP resource URIs, but their top target-file ranking stays
  unchanged.
- Added focused tests for the expanded classifier and the stricter target-file
  floor gate.

## Evidence

Focused tests:

```bash
cargo test -p ctxhelm-compiler product_proof_and_historical_eval_tasks_are_multi_area -- --nocapture
cargo test -p ctxhelm-compiler docs_and_archive_multi_area_tasks_do_not_spend_target_file_source_floors -- --nocapture
```

Broad proof:

```bash
cargo run --release -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > .ctxhelm/e2e/phase98-broader-broad-task-classification-proof.json

python3 scripts/check-product-proof.py \
  .ctxhelm/e2e/phase98-broader-broad-task-classification-proof.json
```

Committed proof:

- `.ctxhelm/e2e/phase98-broader-broad-task-classification-proof.json`

Result:

- `releaseGate.decision = promote`
- RefactoringMiner stays at File Recall@10 `0.6`, Source Recall@10 `1.0`,
  Test Recall@10 `1.0`, Effective Validation Recall@10 `1.0`
- ctxhelm stays at File Recall@10 `0.47460318` and Source Recall@10
  `0.7166667`, matching Phase 97 while adding broad context areas for the two
  newly classified archive/docs tasks
- ctxhelm broad context-area recall remains `1.0`
- ReAgent stays at File Recall@10 `0.5`, Source Recall@10 `1.0`, Test
  Recall@10 `1.0`, Effective Validation Recall@10 `1.0`
- VeriSchema stays at File Recall@10 `0.18449473`, Source Recall@10
  `0.31067252`, Test Recall@10 `0.7089947`, Effective Validation Recall@10
  `1.0`, and broad context-area recall `0.71851856`

## Notes

This phase intentionally does not improve top-10 file recall. Its production
value is preventing wide archive/docs tasks from being silently treated as
single-area work while avoiding the measured regression caused by spending
target-file slots on broad source floors for those tasks.
