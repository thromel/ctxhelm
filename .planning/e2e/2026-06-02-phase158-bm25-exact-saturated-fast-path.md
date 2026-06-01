# Phase 158: BM25 Exact-Saturated Fast Path

## Goal

Fix the Phase 157 RefactoringMiner regression where the active query-time BM25
backend trailed the legacy lexical scanner on a clean 20-commit corpus sample.

## Root Cause

The active BM25 path computed exact lexical evidence for every safe file, but it
only returned files that Tantivy placed inside the top document window. That
discarded high-value exact candidates when broad fielded terms such as `tests`
or `update` dominated the fielded query.

After adding an exact reserve, one row still trailed because BM25 score was used
to reorder exact candidates. For low-information commit titles such as `Update
failing tests`, this let a generic source file nudge a target source file out of
the top-10 budget.

## Implementation

- Bumped the active backend report label to `tantivy_bm25_fielded_v5`.
- Bumped lexical query cache namespace to `lexical-search-cache-v5`.
- Made exact lexical score primary for exact candidates.
- Kept fielded BM25 only for non-exact candidates.
- Added an exact-saturated fast path: when exact candidates already fill the
  requested budget, return the exact ranking without building the query-time
  Tantivy index or extracting symbols.
- Added a focused `ctxhelm-index` regression test for the fast path.

## Direct Failing Row Check

Reproduced parent snapshot for commit `e5e5b38b` and query `Update failing
tests`.

Before the final fix, `src/main/java/gr/uom/java/xmi/diff/UMLAbstractClassDiff.java`
fell outside the active top 10. With v5 it is rank 10, matching the legacy hit
set for that row.

## RefactoringMiner Corpus Proof

Command:

```bash
CTXHELM_HOME=/tmp/ctxhelm-phase158-home-v5 cargo run --release -p ctxhelm --locked -- eval lexical corpus \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 20 \
  --budget 10 \
  --format json > .ctxhelm/e2e/phase158-refactoringminer-lexical-backend-corpus.json
```

Result:

| Metric | Phase 157 BM25 v1 | Phase 158 BM25 v5 | Legacy |
| --- | ---: | ---: | ---: |
| Recall@5 | 0.075 | 0.100 | 0.100 |
| Recall@10 | 0.1125 | 0.175 | 0.175 |
| MRR@10 | 0.11333333 | 0.13916667 | 0.13916667 |
| Backend millis | 63691 | 9671 | 1689 |

Comparison after Phase 158:

- Recall delta@10 versus legacy: `0.0`
- MRR delta@10 versus legacy: `0.0`
- BM25 wins@10: `0`
- Legacy wins@10: `0`
- Ties@10: `20`
- Average overlap@K: `9.7`
- Top path changed rate: `0.0`

## Interpretation

Phase 158 fixes the measured quality regression and reduces cold active-backend
runtime substantially versus Phase 157. It does not make the active backend
faster than the legacy scanner in cold no-cache mode. The next R&D bottleneck is
persistent or reusable fielded indexing for queries that are not exact-saturated.

## Validation

- `cargo fmt --check`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm --locked -- --help`
- `cargo test -p ctxhelm-index lexical_search --locked`
- `cargo test -p ctxhelm --test cli_compat eval_lexical --locked`
- `cargo test --workspace --locked --no-fail-fast`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- RefactoringMiner corpus proof above
