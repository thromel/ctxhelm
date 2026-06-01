# Phase 155 - BM25 Corpus Comparison Report

## Goal

Turn the Phase 154 single-query BM25-vs-legacy diagnostic into a corpus-level
measurement surface. The R&D question is whether the active Tantivy/BM25 lexical
backend improves, regresses, or churns ranking relative to the pre-BM25
heuristic scanner over historical tasks.

## Implementation

- Added `ctxhelm eval lexical corpus`.
  - Arguments: `--repo`, `--limit`, `--budget`, `--base`, `--head`, and
    `--format`.
  - Markdown output is the default; JSON uses typed public contracts.
- Added compiler-level `compare_lexical_backends_on_corpus`.
  - Reuses the historical commit sampler.
  - Evaluates each task against the parent snapshot when a parent revision is
    available.
  - Runs active BM25 lexical ranking and legacy heuristic ranking over the same
    task, repo snapshot, and ranking budget.
- Added source-free aggregate metrics:
  - BM25 and legacy Recall@5/10.
  - BM25 and legacy MRR@10.
  - Recall delta@5/10 and MRR delta@10.
  - Average overlap@K.
  - Top-path changed rate.
  - BM25 win count, legacy win count, and tie count.
  - Backend runtime totals.
- Added source-free per-commit rows:
  - commit SHA;
  - task hash, not task text;
  - retrieval target paths;
  - BM25 and legacy candidate paths;
  - hit lists, overlap, top-path-changed flag, and backend timing.

## Privacy Boundary

The report is intended for R&D and product-proof integration. It does not store
source snippets, raw task text, result reasons, stack traces, terminal logs, or
secrets. Candidate rows contain paths and source-free metrics only.

## Validation

Focused validation:

```bash
cargo fmt --check
cargo test -p ctxhelm --test cli_compat eval_lexical --locked
cargo run -p ctxhelm --locked -- eval lexical corpus --repo . --limit 2 --budget 5 --format json
```

Focused results on the latest two ctxhelm commits:

- BM25 Recall@10: `0.2777778`
- Legacy Recall@10: `0.22222222`
- Recall delta@10: `+0.055555567`
- MRR delta@10: `+0.625`
- Average overlap@K: `3.0`
- Top-path changed rate: `1.0`
- BM25 wins/ties/losses: `1` / `1` / `0`
- BM25/legacy backend time: `2873ms` / `73ms`

This proves the new report can expose both quality lift and a significant
cold-path latency cost for the BM25 backend.

Broader validation should include the standard release stack before push.

## Remaining Work

- Integrate corpus-level BM25-vs-legacy metrics into benchmark/product proof.
- Run the corpus comparison over RefactoringMiner and the broader fixed corpus.
- Use backend win/loss rows to decide whether BM25 scoring needs field boosts,
  exact symbol table promotion, or task-type-specific query construction.
