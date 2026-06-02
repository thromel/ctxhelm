# Phase 172: Benchmarking Governance Doc Priority

## Goal

Close the remaining repeated `ctxhelm` governance-doc gap for `docs/benchmarking.md` without increasing context budget pressure or displacing source evidence.

Phase 171 fixed the dominant root planning-doc order, but the follow-up clean four-repo proof still showed `docs/benchmarking.md` as the top repeated `ctxhelm` miss:

- `docs/benchmarking.md`: missed `4` times
- `crates/ctxpack-compiler/src/ranking.rs`: missed `2` times
- `crates/ctxpack-index/src/lib.rs`: missed `2` times

## Rejected Variant

First experiment: increase the governance-doc floor from `ceil(file_budget / 3).clamp(1, 4)` to `ceil(file_budget / 2).clamp(1, 5)`.

Proof artifact:

`/tmp/ctxhelm-rd/phase172-benchmark-doc-floor-proof.json`

Result:

- Decision: `promote`
- Average File Recall@10: `0.59761477 -> 0.6190433`
- Average ctxhelm lift@10: `+0.14052217 -> +0.16195072`
- Test Recall@10: unchanged
- Rejected because `ctxhelm` Source Recall@10 regressed from `0.46666667` to `0.38333336`

This was an all-file improvement that paid for doc recall by displacing source evidence, so it was not accepted.

## Accepted Change

Keep the existing bounded four-slot governance reserve, but promote `docs/benchmarking.md` inside the root governance priority order:

1. `.planning/STATE.md`
2. `.planning/ROADMAP.md`
3. `.planning/MILESTONES.md`
4. `docs/benchmarking.md`
5. `.planning/REQUIREMENTS.md`
6. `.planning/PROJECT.md`
7. `AGENTS.md`
8. `docs/release.md`
9. `docs/agent-setup.md`
10. `docs/semantic.md`
11. `README.md`

This preserves the existing `governance_doc_floor_limit` and does not add tools, embeddings, reranking, source reads, or budget expansion.

## Proof

Final proof artifact:

`/tmp/ctxhelm-rd/phase172-bounded-benchmark-doc-proof.json`

Baseline proof artifact:

`/tmp/ctxhelm-rd/phase171-governance-priority-final-proof.json`

Config:

`.planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json`

Proof result:

- Decision: `promote`
- Evaluated repositories: `4`
- Average File Recall@10: `0.59761477 -> 0.61190045`
- Average lexical baseline Recall@10: `0.45709258 -> 0.45709258`
- Average ctxhelm lift@10: `+0.14052217 -> +0.15480788`
- Average Test Recall@10: `0.67989415 -> 0.67989415`
- Average brief token ROI: `1.4375 -> 1.4375`

Per-corpus movement:

| Corpus | File Recall@10 before | File Recall@10 after | Source Recall@10 before | Source Recall@10 after | Test Recall@10 before | Test Recall@10 after |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8` | `0.8` | `1.0` | `1.0` | `1.0` | `1.0` |
| ctxhelm | `0.60952383` | `0.6666667` | `0.46666667` | `0.46666667` | `0.0` | `0.0` |
| ReAgent | `0.8` | `0.8` | `1.0` | `1.0` | `1.0` | `1.0` |
| VeriSchema | `0.18093514` | `0.18093514` | `0.2763158` | `0.2763158` | `0.71957666` | `0.71957666` |

Additional `ctxhelm` movement:

- Context Recall@10: `0.6111111 -> 0.6666667`
- Agent Evidence Recall@10: `0.60952383 -> 0.6666667`
- Protected target miss-rate: `0.25 -> 0.16666667`
- `docs/benchmarking.md` dropped out of the `ctxhelm` top-missing list

Remaining `ctxhelm` top misses:

- `.planning/REQUIREMENTS.md`: `2`
- `crates/ctxpack-compiler/src/ranking.rs`: `2`
- `crates/ctxpack-index/src/lib.rs`: `2`
- `crates/ctxpack-compiler/src/planning.rs`: `1`
- `crates/ctxpack-index/src/dependencies.rs`: `1`
- `crates/ctxpack-index/src/related_tests.rs`: `1`
- `crates/ctxpack-index/src/search.rs`: `1`
- `docs/agent-setup.md`: `1`
- `docs/release.md`: `1`

## Validation

Completed during implementation:

- `cargo test -p ctxhelm-compiler selection_prioritizes_current_root_planning_docs_in_governance_floor --locked`
- `cargo run -p ctxhelm --locked -- eval proof --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json --format json`

Full workspace validation is run as closeout.
