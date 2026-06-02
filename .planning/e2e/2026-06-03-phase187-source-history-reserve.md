# Phase 187: Corroborated Source History Reserve

## Goal

Reduce measured source misses where a task-conditioned source file has strong
co-change evidence but is ranked below the fixed top-10 budget.

## Implementation

- Added a bounded source-history reserve to target-file selection.
- The reserve is source-only and does not run when active context anchors are
  present.
- Co-change evidence must be corroborated by another task signal before it can
  claim reserved budget:
  - dependency
  - lexical
  - lexical expansion
  - symbol
- Moved the reserve before broad governance/script expansion can exhaust the
  file budget.
- Added focused regression coverage for standard and broad selection, plus an
  active-anchor guard.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-compiler \
  broad_selection_reserves_source_cochange_targets_when_lexical_fills_budget \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  selection_reserves_source_cochange_targets_when_lexical_fills_budget \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  selection_does_not_let_source_cochange_reserve_displace_active_context_anchor \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler --locked -- --nocapture
```

Result: compiler tests passed with `139 passed`.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase187-source-history-reserve-release-home \
  /tmp/ctxhelm-rd/phase187-source-history-reserve-release-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase187-source-history-reserve-release-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase187-source-history-reserve-release-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase187-source-history-reserve-release-proof.json
```

Result:

- `releaseGate.decision = promote`
- RefactoringMiner unchanged:
  - File Recall@10 `0.8`
  - Source Recall@10 `1.0`
- ctxhelm source recall improved, with an all-file tradeoff:
  - File Recall@10 `0.6666667 -> 0.5587302`
  - Source Recall@10 `0.42857143 -> 0.5`
  - Source hits `6 -> 7`
- ReAgent unchanged:
  - File Recall@10 `0.8`
  - Source Recall@10 `1.0`
- VeriSchema source recall improved:
  - File Recall@10 `0.35529414`
  - Source Recall@10 `0.32258064 -> 0.38709676`
  - Source hits `10 -> 12`

## Tradeoff

This is not an all-file recall win. It is a source-channel improvement for the
measured co-change/dependency source-pressure gap family. On ctxhelm, broad
docs lose some top-10 budget while source hits improve. The gate still promotes
because non-test context recall beats lexical and source recall does not
regress, but future ranking work should look for a better balanced allocation
between broad docs and corroborated source-history candidates.
