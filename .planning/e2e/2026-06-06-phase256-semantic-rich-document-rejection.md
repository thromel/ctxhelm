# Phase 256 - Semantic Rich-Document Rejection

## Goal

Test whether `local_fastembed` should use richer source-free semantic search
documents with symbol and dependency facets, rather than the current bounded
metadata/path-only search documents.

## Experiment

Temporarily changed semantic search and semantic vector indexing to:

- include source-free symbol facets;
- include source-free dependency facets;
- bump semantic document text version so stale metadata-only vectors would not
  be reused.

The change preserved source-free behavior in unit tests, but the quality gate
decides whether it is worth shipping.

## Evidence

Source-free artifacts:

- `.ctxhelm/e2e/phase256-semantic-gate-refactoringminer-before.json`
- `.ctxhelm/e2e/phase256-semantic-gate-refactoringminer-after.json`
- `.ctxhelm/e2e/phase256-semantic-gate-ctxhelm-before.json`
- `.ctxhelm/e2e/phase256-semantic-gate-ctxhelm-after.json`

Gate comparison:

| Repo | Variant | Before Recall@10 | After Recall@10 | Before Runtime | After Runtime |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `ctxhelm_default` | `0.5383333` | `0.5383333` | `54065ms` | `8840ms` |
| RefactoringMiner | `local_semantic` | `0.5583333` | `0.5383333` | `14457ms` | `101561ms` |
| RefactoringMiner | `local_metadata_reranked` | `0.6166667` | `0.6166667` | `7772ms` | `7728ms` |
| ctxhelm | `ctxhelm_default` | `0.31237346` | `0.31237346` | `23162ms` | `1636ms` |
| ctxhelm | `local_semantic` | `0.31237346` | `0.31237346` | `9938ms` | `23837ms` |
| ctxhelm | `local_metadata_reranked` | `0.5595388` | `0.5595388` | `1662ms` | `1971ms` |

Privacy checks found no `sourceTextLogged`, raw prompt, remote embedding, or
remote reranking regressions in the after artifacts.

## Decision

Rejected and reverted.

Richer symbol/dependency semantic search documents are source-free, but they do
not improve the current `local_fastembed` default-lift question:

- RefactoringMiner loses the small semantic lift from Phase 252/256 before.
- ctxhelm remains neutral.
- Runtime worsens materially for both semantic gates.

This narrows the next semantic R&D step. Do not keep adding facets to
`local_fastembed` search documents by default. The better near-term direction is
to investigate task-conditioned semantic query construction, local metadata
reranker promotion safety, or alternate local models/fusion, while keeping
semantic opt-in until a gate proves repeated lift without runtime or protected
evidence regressions.
