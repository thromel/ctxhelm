# Phase 191: Context Area Pressure Breakdown

## Goal

Make Phase 190's `inspectionPressure` score explainable. A bare pressure value
helps agents prioritize broad areas, but it does not say whether the unread
surface is implementation, validation, or docs pressure. This phase adds a
source-free breakdown so agents and proof artifacts can explain why an area is
high pressure without loading source text or changing the top-10 budget.

## Implementation

- Added `inspectionPressureBreakdown` to plan-level `contextAreas`.
  - `sourceLikeUnselected`
  - `validationUnselected`
  - `docsUnselected`
  - role weights
  - weighted `total`
- Kept `inspectionPressure` equal to the breakdown total.
- Rendered the breakdown in context-pack area guidance.
- Propagated the breakdown into retrieval-gap summaries when a matching
  task-conditioned context-area profile is available.
- Extended `scripts/check-product-proof.py` to validate emitted context-area
  pressure fields and weighted totals.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-core context_plan_public_json_shape_is_stable --locked -- --nocapture
cargo test -p ctxhelm-compiler context_areas --locked -- --nocapture
cargo test -p ctxhelm-compiler retrieval_gap_summaries_skip_validation_covered_tests --locked -- --nocapture
cargo test -p ctxhelm product_proof_checker_accepts_promote_and_rejects_block --locked -- --nocapture
```

Result: passed.

Release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase191-area-pressure-breakdown-home \
  /tmp/ctxhelm-rd/phase191-area-pressure-breakdown-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase191-area-pressure-breakdown-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase191-area-pressure-breakdown-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase191-area-pressure-breakdown-proof.json
```

Result:

- `releaseGate.decision = promote`
- `contextAreas = 144`
- `badBreakdowns = 0`
- All four repositories keep Phase 190 file/source/test/context-area metrics.

Metric comparison against Phase 190:

| Repo | File Recall@10 | Source Recall@10 | Test Recall@10 | Broad Area Recall |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8 -> 0.8` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| ctxhelm | `0.67777777 -> 0.67777777` | `0.55 -> 0.55` | `0.0 -> 0.0` | `1.0 -> 1.0` |
| ReAgent | `0.8 -> 0.8` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| VeriSchema | `0.35529414 -> 0.35529414` | `0.5277778 -> 0.5277778` | `0.7896825 -> 0.7896825` | `0.5777778 -> 0.5777778` |

## Why It Matters

This turns context-area pressure from a ranking hint into an inspectable
diagnostic. Agents can now tell whether to spend their next native reads on
source/config/schema files, validation tests, or docs, while the release proof
guards the arithmetic and keeps source-free output contracts intact.
