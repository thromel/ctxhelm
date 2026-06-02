# Phase 190: Context Area Inspection Pressure

## Goal

Improve broad-area guidance without changing the fixed top-10 target-file
budget. Phase 189 showed that remaining misses are often inside surfaced
context areas rather than completely invisible to retrieval. Agents need a
source-free way to prioritize which surfaced area still has the most unread
surface.

## Implementation

- Added `coveragePercent` to plan-level `contextAreas`.
  - Computed as selected paths divided by candidate paths.
  - Source-free and derived only from safe inventory/candidate metadata.
- Added `inspectionPressure` to plan-level `contextAreas`.
  - Weights unselected source/config/schema paths higher than tests, and tests
    higher than docs.
  - Gives agents a compact "under-read area" signal without source text or gold
    labels.
- Rendered coverage and pressure in context-pack area guidance.
- Ordered zero-selected pack guidance by inspection pressure so the next native
  reads focus on the largest still-unread surfaced areas first.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-core context_plan_public_json_shape_is_stable --locked -- --nocapture
cargo test -p ctxhelm-compiler context_areas_include_docs_and_next_read_paths --locked -- --nocapture
cargo test -p ctxhelm-compiler compile_context_pack_renders_context_areas --locked -- --nocapture
```

Result: passed.

Full release gate:

```bash
CTXHELM_ALLOW_DIRTY=1 bash scripts/release-gate.sh
```

Result: passed. This included workspace tests, release docs consistency,
release packaging/audit, storage/memory/feedback/workspace/product smokes,
wrong-cwd MCP protocol proof, Cursor/OpenCode setup protocol evidence, and the
Codex/Claude deterministic protocol gates.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase190-context-area-pressure-home \
  /tmp/ctxhelm-rd/phase190-context-area-pressure-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase190-context-area-pressure-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase190-context-area-pressure-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase190-context-area-pressure-proof.json
```

Result:

- `releaseGate.decision = promote`
- All four repositories keep Phase 189 file/source/test/context-area metrics.
- Every sampled `contextAreas` entry includes `coveragePercent` and
  `inspectionPressure`.

Metric comparison against Phase 189:

| Repo | File Recall@10 | Source Recall@10 | Broad Area Recall |
| --- | ---: | ---: | ---: |
| RefactoringMiner | `0.8 -> 0.8` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| ctxhelm | `0.67777777 -> 0.67777777` | `0.55 -> 0.55` | `1.0 -> 1.0` |
| ReAgent | `0.8 -> 0.8` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| VeriSchema | `0.35529414 -> 0.35529414` | `0.5277778 -> 0.5277778` | `0.5777778 -> 0.5777778` |

## Why It Matters

This makes broad-area guidance more actionable without spending more top-10
budget or tuning global thresholds. It is a step toward ctxhelm acting as a
context governor: the agent can see not only which areas were surfaced, but
which surfaced areas are still under-read and worth progressive native reads.
