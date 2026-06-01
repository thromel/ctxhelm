# Phase 154 - BM25 Legacy Comparison Report

## Goal

Measure the R&D memo's BM25 lexical-index bet against the pre-BM25 scanner
without changing production ranking again. Phase 153 made BM25 the active
lexical path; Phase 154 adds the missing source-free measurement surface for
backend-level comparison.

## Implementation

- Added `ctxhelm eval lexical compare`.
  - Arguments: `--repo`, `--query`, `--limit`, and `--format`.
  - Markdown output is the default.
  - JSON output uses schema `ctxhelm-lexical-comparison-v1`.
- Added a comparison-only legacy lexical path in `ctxhelm-index`.
  - It reuses the previous heuristic file scanner.
  - It does not persist comparison results.
  - It respects the same safe inventory, generated-file, sensitive-file, and
    ignored-file exclusions as normal search.
- Compared the active BM25 backend with the legacy scanner.
  - Records overlap@limit.
  - Records BM25-only paths and legacy-only paths.
  - Records BM25 and legacy top paths plus whether the top path changed.
  - Records source-free backend diagnostics by code/severity count.
- Kept the report source-free.
  - Raw query text is replaced by a query hash.
  - Repo path is replaced by a path hash plus local label.
  - Result rows contain path, role, language, and score only.
  - Result reasons are intentionally omitted because reasons can echo query
    terms.

## Validation

Focused validation:

```bash
cargo fmt --check
cargo test -p ctxhelm-index legacy_lexical --locked
cargo test -p ctxhelm --test cli_compat eval_lexical --locked
```

Passed before the broader validation gate.

## Remaining Work

- Run this backend comparison across the fixed corpora and native-agent outcome
  suites rather than only single-query smoke coverage.
- Decide whether the existing lexical-search cache should gain a stricter
  source-free/no-query-term report mode before persistent BM25 storage is
  introduced.
- Add recall/latency deltas from BM25-vs-legacy comparisons into product proof
  once the comparison is corpus-aware.
