---
phase: 62
title: Production Local Embedding Quality
status: complete
completed: 2026-05-30
requirements_addressed:
  - EMBED-01
  - EMBED-02
  - EMBED-03
  - EMBED-04
---

# Phase 62 Summary: Production Local Embedding Quality

## Outcome

Phase 62 is complete. `local_fastembed` is usable as a source-free,
production-local embedding provider behind the `local-embeddings` feature, but it
is not promoted as a default retrieval signal.

Decision: hold `local_fastembed` as opt-in. It matched default Recall@10 on the
two-repo corpus but took much longer than default retrieval and did not beat the
lexical baseline.

## Implementation

- Benchmark suite configs now accept `semanticProvider`, `semanticModel`, and
  `semanticDimensions`.
- Benchmark reports now resolve effective semantic provider metadata:
  `semanticProvider`, `semanticModel`, `semanticDimensions`,
  `semanticProviderRole`, and `semanticQualityBackend`.
- `local_hash` reports as `deterministic_scaffold` with
  `qualityBackend=false`.
- `local_fastembed` reports as `production_local` with `qualityBackend=true`.
- Fastembed model cache defaults to repo `.ctxhelm/cache/fastembed` inside a git
  repo, otherwise `CTXHELM_HOME/cache/fastembed`, and can be overridden with
  `CTXHELM_FASTEMBED_CACHE_DIR`.
- Fastembed source-free vector reuse is bounded in process.
- Fastembed query-time candidate embedding is bounded by
  `CTXHELM_FASTEMBED_DOCUMENT_LIMIT`, default `128`.
- `.ctxhelm/cache/` and `.fastembed_cache/` are excluded from repo inventory and
  git tracking.

## Evidence

Source-free E2E summary:

- `.planning/e2e/2026-05-30-phase62-production-local-embedding-quality.md`

Ignored large JSON artifacts:

- `.ctxhelm/e2e/phase62-default-report.json`
- `.ctxhelm/e2e/phase62-local-hash-report.json`
- `.ctxhelm/e2e/phase62-local-fastembed-report.json`
- `.ctxhelm/e2e/phase62-local-fastembed-status.json`
- `.ctxhelm/e2e/phase62-local-fastembed-minilm-status.json`

## Results

| Variant | RefactoringMiner Recall@10 | ctxhelm Recall@10 | Total repo runtime |
| --- | ---: | ---: | ---: |
| Default, semantic off | 0.7767 | 0.2299 | 26.3s |
| `local_hash` | 0.7767 | 0.2299 | 57.8s |
| `local_fastembed` `AllMiniLML6V2Q` | 0.7767 | 0.2299 | 183.7s |

## Follow-Up

Phase 63 should focus on promotion-safe reranking/fusion before attempting any
semantic default promotion. Phase 64 should target repeated gap families because
embedding alone did not improve the fixed corpus.
