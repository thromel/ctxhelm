# Phase 196: Validation Context Area Reserve

## Goal

Phase 195 expanded `nextReadPaths` inside high-pressure areas, but the
VeriSchema proof still showed validation misses that were already present in
`recommendedTests` yet absent from surfaced `contextAreas`. That made broad
progressive reads source-heavy even when validation clusters were the strongest
second-step evidence for an agent.

## Implementation

- Added package-mirror affinity to related-test scoring:
  - `schema_agent/agents/base.py` can now map to `tests/agents/...`
  - `src/main/java/...` can now map to `src/test/java/...`
  - `src/main/kotlin/...` can now map to `src/test/kotlin/...`
- Added focused related-test coverage for Python package mirrored tests.
- Added broad context-area sorting that reserves selected validation areas for
  broad tasks before source-only tail areas can truncate them away.
- Kept focused context-area filtering unchanged.
- Added focused planner coverage proving a selected `tests/agents` area remains
  surfaced even when more than sixteen source areas compete for the area budget.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-index \
  related_tests_prefers_python_package_mirrored_test_directory \
  --locked -- --nocapture
cargo test -p ctxhelm-index \
  related_tests_uses_gradle_java_test_class_command \
  --locked -- --nocapture
cargo test -p ctxhelm-compiler \
  broad_context_areas_reserve_selected_validation_areas \
  --locked -- --nocapture
cargo fmt --all -- --check
```

Result: passed.

Rejected intermediate proof:

```bash
/tmp/ctxhelm-rd/phase196-mirrored-test-affinity-proof.json
```

Result: `releaseGate.decision = promote`, but broad proof metrics were unchanged
from Phase 195. The direct `related-tests` tool improved, but the progressive
context pack path did not. That was not enough to count as the R&D improvement.

Accepted release-binary product proof:

```bash
rm -rf /tmp/ctxhelm-phase196-validation-area-home \
  /tmp/ctxhelm-rd/phase196-validation-area-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase196-validation-area-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase196-validation-area-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase196-validation-area-proof.json
```

Result: `releaseGate.decision = promote`.

Selected-file metrics stayed unchanged from Phase 195:

| Repo | File Recall@10 | Source Recall@10 | Test Recall@10 | Effective Validation Recall@10 |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8` | `1.0` | `1.0` | `1.0` |
| ctxhelm | `0.67777777` | `0.55` | `0.0` | `0.0` |
| ReAgent | `0.8` | `1.0` | `1.0` | `1.0` |
| VeriSchema | `0.35529414` | `0.5277778` | `0.7896825` | `1.0` |

Progressive broad-area metrics improved on VeriSchema:

| Repo | Phase 195 Broad Area Recall | Phase 196 Broad Area Recall | Phase 195 Next-Read Recovery | Phase 196 Next-Read Recovery |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.0` | `0.0` | `0 / 1` | `0 / 1` |
| ctxhelm | `1.0` | `1.0` | `11 / 12` | `11 / 12` |
| ReAgent | `0.0` | `0.0` | `0 / 0` | `0 / 0` |
| VeriSchema | `0.5777778` | `0.84444445` | `16 / 39` | `19 / 39` |

## Why It Matters

This is a context-compiler improvement rather than a top-10 ranking change. The
initial selected file budget stays stable, but broad task packs now expose
validation clusters as progressive context areas when tests are already selected
by the planner. That gives agents a better second step for wide changes:
inspect the related validation area instead of continuing through source-only
tail areas.
