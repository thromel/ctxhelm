# Phase 171: Governance Doc Priority

## Goal

Improve retrieval for eval/release/history tasks where current project-state docs are changed together with implementation code but are ranked below older or generic docs.

The measured gap came from the clean four-repo proof after Phase 170:

- ctxhelm repeated misses: `.planning/STATE.md`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `docs/benchmarking.md`
- Dominant gap family: `ranked_below_budget_co_change_lexical_expansion_docs`
- Recommendation area: `historyRanking`

## Change

Governance-doc floors now rank current root planning docs with stable product-state priority before lexical score:

1. `.planning/STATE.md`
2. `.planning/ROADMAP.md`
3. `.planning/MILESTONES.md`
4. `.planning/REQUIREMENTS.md`
5. `.planning/PROJECT.md`
6. `AGENTS.md`
7. `docs/benchmarking.md`
8. `docs/release.md`
9. `docs/agent-setup.md`
10. `docs/semantic.md`
11. `README.md`

This keeps the existing governance-doc floor bounded. It does not add tools, increase the context budget, inspect source content, or alter validation-test reservation.

## Rejected Work

A separate nested broad-source family ordering experiment was tested first. It passed a focused unit test but produced no selected-context change and no four-repo metric movement, so it was removed before final validation.

## Proof

Final proof artifact:

`/tmp/ctxhelm-rd/phase171-governance-priority-final-proof.json`

Config:

`.planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json`

Proof result:

- Decision: `promote`
- Evaluated repositories: `4`
- Evaluated commits: `16`
- Average File Recall@10: `0.5499957 -> 0.59761477`
- Average lexical baseline Recall@10: `0.45709258 -> 0.45709258`
- Average ctxhelm lift@10: `+0.09290312 -> +0.14052217`
- Average Test Recall@10: `0.67989415 -> 0.67989415`
- Average brief token ROI: `1.375 -> 1.4375`

Per-corpus movement:

| Corpus | File Recall@10 before | File Recall@10 after | Context Recall@10 before | Context Recall@10 after |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8` | `0.8` | `1.0` | `1.0` |
| ctxhelm | `0.41904765` | `0.60952383` | `0.41666666` | `0.6111111` |
| ReAgent | `0.8` | `0.8` | `1.0` | `1.0` |
| VeriSchema | `0.18093514` | `0.18093514` | `0.23287672` | `0.23287672` |

Representative ctxhelm context movement:

- `7f439fa1b`: top-10 replaced lower-value `.planning/PROJECT.md`, `README.md`, and extra release docs with `.planning/STATE.md`, `.planning/ROADMAP.md`, and `.planning/REQUIREMENTS.md`; missing count dropped from `6` to `3`.
- `bd5e0468c`: missing count dropped from `2` to `1`.
- `4e950d58c`: missing count dropped from `4` to `2`.

## Validation

Completed before closeout:

- `cargo fmt --check`
- `cargo test -p ctxhelm-compiler selection_prioritizes_current_root_planning_docs_in_governance_floor --locked`
- `cargo run -p ctxhelm --locked -- eval proof --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json --format json`

Full workspace validation is run as closeout.
