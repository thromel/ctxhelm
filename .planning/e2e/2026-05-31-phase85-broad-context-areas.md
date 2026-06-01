# Phase 85: Broad Context Areas

## Goal

Broad workflow, lint, release, and eval tasks often touch multiple repository
areas. Phase 84 preserved bounded dependency source evidence for those tasks,
but the first `prepare-task` response still forced agents to infer adjacent
areas from the target-file list alone. Phase 85 adds an additive area-level
diagnostic channel so agents can inspect likely adjacent areas without
displacing protected target files, tests, or validation commands.

## Change

- Added a typed `ContextArea` contract to `ContextPlan`.
- Populated `contextAreas` only when the task is classified as broad
  multi-area.
- Grouped source, test, config, and schema candidates by repository area and
  reported candidate count, selected count, and up to three representative
  paths.
- Rendered `contextAreas` in generated packs after risk flags.
- Documented the field as source-free, additive, and outside the target-file
  recall budget.

Example local `prepare-task` query:

```bash
cargo run -p ctxhelm -- prepare-task "stabilize lint workflow" --mode bug-fix --no-trace
```

The response included `multi_area_task` plus context areas for:

- `crates/ctxhelm-compiler`: 10 candidate paths, 5 selected paths
- `crates/ctxhelm`: 4 candidate paths, 3 selected paths
- `crates/ctxhelm-mcp`: 6 candidate paths, 2 selected paths
- `crates/ctxhelm-index`: 4 candidate paths, 0 selected paths
- `crates/ctxhelm-core`: 1 candidate path, 0 selected paths

## Rejected Ranking Experiments

These experiments were intentionally not kept because they changed protected
top-10 evidence or failed to improve the target gap:

- Broad parent-bounded co-change source ranking worsened VeriSchema Source
  Recall@10 from `0.304` to `0.291`.
- Dependency floor max 5 improved VeriSchema Source Recall@10 from `0.304` to
  `0.325`, but worsened protected target miss-rate from `0.286` to `0.571`.
- Dependency floor max 4 did not improve VeriSchema Source Recall@10 and still
  worsened protected target miss-rate to `0.429`.
- Classifying shell scripts as config reduced VeriSchema File Recall@10 from
  `0.179` to `0.174`.
- Depth-2 graph expansion worsened VeriSchema File Recall@10 from `0.179` to
  `0.157` and Source Recall@10 from `0.304` to `0.263`.

The retained Phase 85 change is deliberately non-displacing guidance rather
than another ranking perturbation.

## Proof

### Focused Behavior

- `cargo test -p ctxhelm-core context_plan_public_json_shape_is_stable -- --nocapture`
- `cargo test -p ctxhelm-core retrieval_contracts_serialize_additive_camel_case_fields -- --nocapture`
- `cargo test -p ctxhelm-core workspace_context_plan_contract_is_source_free -- --nocapture`
- `cargo test -p ctxhelm-compiler prepare_context_plan_reports_multi_area_task_diagnostics -- --nocapture`

### Broad Fixed-Corpus Cold Proof

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxhelm-phase85-context-areas-proof.json
```

The cold proof blocked only on the existing local runtime gate:

```text
Blocked because proof runtime exceeded 5000ms per commit for: RefactoringMiner.
```

Quality movement was intentionally flat versus Phase 84 because `contextAreas`
does not alter ranking:

- RefactoringMiner: unchanged.
- ctxhelm: unchanged.
- ReAgent: unchanged.
- VeriSchema File Recall@10 stayed `0.17936651`.
- VeriSchema Source Recall@10 stayed `0.30409357`.
- VeriSchema Test Recall@10 stayed `0.7089947`.
- VeriSchema protected target miss-rate stayed `0.2857143`.

### Required Two-Repo Cold Proof

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .ctxhelm/e2e/v25-multirepo-baseline-config.json \
  --format json > /tmp/ctxhelm-phase85-context-areas-required-proof.json
```

This proof also blocked only on the existing RefactoringMiner runtime gate, not
quality:

- RefactoringMiner context Recall@10 `0.7777778`, Effective Validation
  Recall@10 `1.0`, protected target miss-rate `0.0`.
- ctxhelm context Recall@10 `0.68421054`, Effective Validation Recall@10
  `1.0`, protected target miss-rate `0.027027028`.

### Warm-Cache Proof

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json \
  --format json > /tmp/ctxhelm-phase85-context-areas-warm-cache-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase85-context-areas-warm-cache-proof.json
```

The warm-cache proof promotes and is committed at:

```text
.ctxhelm/e2e/phase85-context-areas-warm-proof.json
```

Runtime evidence:

- RefactoringMiner: cache hit, `4ms`
- ctxhelm: cache hit, `7ms`
- ReAgent: cache hit, `13ms`
- VeriSchema: cache hit, `8ms`

## Outcome

Phase 85 improves broad-task agent guidance while keeping target-file, test,
validation, and protected-evidence channels stable. It does not claim a recall
lift; the production value is that agents get source-free adjacent-area hints
without a larger or noisier context pack.

## Next Work

- Add task-aware area diversity only if proof shows it can improve misses
  without protected target regressions.
- Continue parser/precision work for remaining `no_candidate_signal` families.
- Continue test-mapping work where validation commands still cover more than
  raw test retrieval.
