# Phase 189: Balanced Broad History And Governance Budget

## Goal

Fix the Phase 187 source-vs-doc budget tradeoff using the Phase 188 selected
signal profiles. Broad ctxhelm proof tasks were spending several top-10 slots on
co-change source candidates while root planning/release docs that were retrieval
targets lost budget.

## Implementation

- Moved broad-scope root governance docs ahead of the source-history reserve.
- Kept source-history reserve early for standard/narrow tasks.
- Kept broad source-history reserve active when no root governance docs compete
  for the fixed top-10 budget.
- Ordered eligible source-history candidates to prefer module/package
  entrypoints such as:
  - `src/lib.rs`
  - `src/main.rs`
  - `src/mod.rs`
  - `__init__.py`
  - `index.ts/js` variants
- Added focused regression coverage for:
  - broad source-history reserve without governance pressure
  - broad governance docs before source-history reserve
  - source-history entrypoint priority

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-compiler \
  broad_selection_reserves_source_cochange_targets_when_lexical_fills_budget \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  broad_selection_keeps_governance_docs_before_source_cochange_reserve \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  source_history_floor_prefers_module_entrypoints \
  --locked -- --nocapture
```

Result: passed.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase189-balanced-broad-history-home \
  /tmp/ctxhelm-rd/phase189-balanced-broad-history-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase189-balanced-broad-history-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase189-balanced-broad-history-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase189-balanced-broad-history-proof.json
```

Result:

- `releaseGate.decision = promote`
- RefactoringMiner unchanged:
  - File Recall@10 `0.8`
  - Source Recall@10 `1.0`
- ctxhelm improves while preserving Phase 187/188 source recall:
  - File Recall@10 `0.5587302 -> 0.67777777`
  - Source Recall@10 `0.55 -> 0.55`
- ReAgent unchanged:
  - File Recall@10 `0.8`
  - Source Recall@10 `1.0`
- VeriSchema unchanged:
  - File Recall@10 `0.35529414`
  - Source Recall@10 `0.5277778`

## Why It Matters

This is the first pass that uses the Phase 188 selected-signal profiles to make
a ranking allocation change. It improves the broad ctxhelm all-file gap without
giving back the Phase 187 source recall gain, and keeps the change source-free:
it only reorders already-eligible governance docs and corroborated source-history
candidates.
