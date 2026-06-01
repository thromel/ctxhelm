# Phase 159: Lexical Runtime Accounting And Exact-Primary Policy

## Goal

Close the remaining Phase 158 runtime gap between the active fielded BM25
lexical backend and the legacy heuristic scanner without losing the clean
RefactoringMiner recall parity established by Phase 158.

## Root Cause

Phase 158 fixed quality by making exact lexical evidence primary, but two
runtime problems remained:

- Corpus timing charged active search for shared inventory warmup before timing
  legacy search, so backend totals were not fully comparable.
- A single exact-identifier row, `77760c1a` / `Improvement in
  TypeScriptVisitor`, still built the full query-time Tantivy index even though
  exact lexical evidence already produced the same useful file set as legacy.

That row dominated backend time: before the final policy fix, it took roughly
`5802ms` active time versus `51ms` legacy time.

## Implementation

- Added `runtime.inventoryWarmupMillis` to the lexical corpus comparison report
  and CLI markdown rendering.
- Warmed parent-snapshot inventory once before timing either backend.
- Added a source-safe `inventoryFingerprint` to inventory metadata and bumped
  inventory schema version to `3`.
- Switched lexical result cache and in-process fielded index cache keys to use
  the inventory fingerprint instead of repeatedly hashing file metadata at
  query time.
- Added a bounded in-process fielded BM25 index cache for repeated non-exact
  queries against the same inventory snapshot.
- Added an exact-dominant fast path: when exact candidates already form a strong
  result set, return them without building the fielded index.
- Added a single-identifier fast path for long exact identifier queries such as
  `TypeScriptVisitor`.
- Added generic task verbs such as `improvement` to the low-value query-term
  filter.
- Added focused regression tests for exact-saturated and exact-dominant fast
  paths.

## RefactoringMiner Corpus Proof

Command:

```bash
CTXHELM_HOME=/tmp/ctxhelm-phase159-home-v8-fresh cargo run --release -p ctxhelm --locked -- eval lexical corpus \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 20 \
  --budget 10 \
  --format json > .ctxhelm/e2e/phase159-refactoringminer-lexical-backend-runtime.json
```

Result:

| Metric | Phase 157 BM25 v1 | Phase 158 BM25 v5 | Phase 159 BM25 v6 | Legacy |
| --- | ---: | ---: | ---: | ---: |
| Recall@5 | 0.075 | 0.100 | 0.100 | 0.100 |
| Recall@10 | 0.1125 | 0.175 | 0.175 | 0.175 |
| MRR@10 | 0.11333333 | 0.13916667 | 0.13916667 | 0.13916667 |
| Backend millis | 63691 | 9671 | 1688 | 1680 |

Phase 159 comparison:

- Recall delta@10 versus legacy: `0.0`
- MRR delta@10 versus legacy: `0.0`
- BM25 wins@10: `0`
- Legacy wins@10: `0`
- Ties@10: `20`
- Average overlap@K: `9.65`
- Top path changed rate: `0.0`
- Shared inventory warmup: `1304ms`

The formerly slow `77760c1a` row now takes `28ms` active time versus `27ms`
legacy time and returns the same three files.

## Interpretation

Phase 159 turns the BM25 integration from "quality parity but too slow" into
"quality parity with legacy-runtime parity" on the clean RefactoringMiner
20-commit proof. This does not prove BM25 beats legacy yet. It proves the active
backend no longer pays a large avoidable penalty for exact-heavy repository
tasks, which makes future semantic/graph/fielded ranking experiments cheaper and
fairer to evaluate.

## Validation

- `cargo fmt --check`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm --locked -- --help`
- `cargo test -p ctxhelm-index lexical_search --locked`
- `cargo test -p ctxhelm-index inventory_metadata_records_safe_file_manifest --locked`
- `cargo test -p ctxhelm --test cli_compat eval_lexical --locked`
- `cargo test --workspace --locked --no-fail-fast`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `git diff --check`
- RefactoringMiner corpus proof above
