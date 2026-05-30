# Phase 88: Broad Source-Area Candidates

## Goal

Reduce remaining broad-task source `no_candidate_signal` gaps without repeating
the rejected broad source-area diversity selector. The Phase 87 proof showed
that broad VeriSchema commits still had source files in areas such as
`schema_agent/nlp`, `schema_agent/algorithms`, and `schema_agent/text2r` that
never appeared as retrieval candidates.

## Change

- Broad multi-area tasks now add bounded source-area inventory candidates from
  the same source root as real lexical hits.
- The candidates are inserted after graph and related-test seed selection, so
  they can affect rank/selection and context-area visibility without expanding
  dependency or test-mapping seeds.
- Auxiliary source roots such as `examples`, `tests`, `docs`, and `scripts` are
  excluded.
- The candidate score is intentionally controlled: strong enough to enter the
  existing source lexical floor, but still below true lexical/symbol/dependency
  evidence.

## Evidence

Focused tests:

```bash
cargo test -p ctxpack-compiler broad_source_area_candidates -- --nocapture
```

Pinned broader proof:

```bash
cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxpack-phase88-broad-source-area-candidates-proof-v3.json
```

Committed artifact:

```text
.ctxpack/e2e/phase88-broad-source-area-candidates-proof.json
```

## Result

Quality movement versus Phase 87:

| Corpus | File Recall@10 | Source Recall@10 | Test Recall@10 | Effective Validation Recall@10 | Protected Target Miss-Rate@10 |
| --- | --- | --- | --- | --- | --- |
| RefactoringMiner | `0.6 -> 0.6` | `1.0 -> 1.0` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| ctxpack | `0.44603175 -> 0.44603175` | `0.6333333 -> 0.6333333` | `0.0 -> 0.0` | `0.0 -> 0.0` | `0.0 -> 0.0` |
| ReAgent | `0.5 -> 0.5` | `1.0 -> 1.0` | `1.0 -> 1.0` | `1.0 -> 1.0` | `0.0 -> 0.0` |
| VeriSchema | `0.17936651 -> 0.18449473` | `0.30409357 -> 0.31067252` | `0.7089947 -> 0.7089947` | `1.0 -> 1.0` | `0.2857143 -> 0.2857143` |

The proof still blocks on the known cold runtime threshold:

```text
Blocked because proof runtime exceeded 5000ms per commit for: RefactoringMiner, ctxpack.
```

That is the existing optional broad-proof runtime blocker. The retrieval change
improves VeriSchema source/file recall without regressing raw test recall,
effective validation recall, or protected retrieval-target miss-rate.

## Rejected Variant

An earlier version inserted broad source-area candidates before graph/test seed
selection. It improved VeriSchema File Recall@10 from `0.17936651` to
`0.18754148` and Source Recall@10 from `0.30409357` to `0.33260235`, but raw
Test Recall@10 dropped from `0.7089947` to `0.6613757`. That version was
rejected because candidate generation should not perturb validation-test
selection.

## Next Work

- Continue reducing remaining source `no_candidate_signal` families with
  candidate-generation fixes that do not expand validation seeds.
- Investigate the remaining `schema_agent/core`, `schema_agent/nlp`, and
  nested `schema_agent/text2r/normalizers` misses.
- Keep the broader proof optional until cold runtime thresholds are addressed.
