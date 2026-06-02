# Phase 188: Selected Signal Profiles

## Goal

Make ranking-budget tradeoffs explainable at the commit level without logging
source text. Phase 187 improved source recall but displaced some broad docs; the
next R&D step needs evidence showing which signal families and roles occupy the
top-10 context budget.

## Implementation

- Added `selectedSignalProfiles` to each historical commit eval report.
- Each profile row is source-free and records:
  - retrieval signal
  - file role
  - selected top-10 count
  - selected top-10 retrieval-target count
- Kept the field local to historical eval reporting; MCP tools and context pack
  output are unchanged.
- Added focused unit coverage for role/signal aggregation and retrieval-target
  hit accounting.

## Proof

Focused test:

```bash
cargo test -p ctxhelm-compiler \
  selected_signal_profiles_count_roles_signals_and_target_hits \
  --locked -- --nocapture
```

Result: passed.

Full compiler tests:

```bash
cargo test -p ctxhelm-compiler --locked -- --nocapture
```

Result: `140 passed`.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase188-selected-signal-profile-home \
  /tmp/ctxhelm-rd/phase188-selected-signal-profile-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase188-selected-signal-profile-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase188-selected-signal-profile-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase188-selected-signal-profile-proof.json
```

Result:

- `releaseGate.decision = promote`
- All 16 evaluated commits have non-empty `selectedSignalProfiles`.
- Retrieval metrics are intentionally unchanged from Phase 187:
  - RefactoringMiner File Recall@10 `0.8`, Source Recall@10 `1.0`
  - ctxhelm File Recall@10 `0.5587302`, Source Recall@10 `0.55`
  - ReAgent File Recall@10 `0.8`, Source Recall@10 `1.0`
  - VeriSchema File Recall@10 `0.35529414`, Source Recall@10 `0.5277778`

## Why It Matters

The new profile shows, per commit, how many selected top-10 files came from
lexical, lexical expansion, dependency, co-change, docs, and other signals, split
by role. For the current ctxhelm broad-task tradeoff, this exposes source-free
evidence such as co-change source slots carrying retrieval-target hits while
some docs are still displaced. That makes the next ranking allocation change
measurable instead of heuristic-only.
