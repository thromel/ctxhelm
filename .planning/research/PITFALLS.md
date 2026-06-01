# Pitfalls Research: v2.4 Production Semantic & Precision Backends

## Pitfall 1: Treating Real Embeddings As Automatic Lift

The May 19 ablation shows that adding a semantic signal can change candidates and slow the run without improving Recall@10. v2.4 must treat embeddings as a hypothesis to test, not a default to celebrate.

Mitigation:

- paired fixed-corpus evals
- minimum lift thresholds
- runtime budget
- rollback/default-blocking verdicts

## Pitfall 2: Embedding Weak Semantic Documents

Code retrieval needs identifiers, paths, signatures, imports, tests, and graph context. Embedding generic chunks can underperform exact search.

Mitigation:

- build typed semantic documents
- include symbol/test/precision metadata
- emit source-free construction traces
- compare document variants in ablations

## Pitfall 3: Precision Indexer Fragility

SCIP/LSP generation can depend on build tools, dependency installation, language servers, and repo setup. Making it mandatory would violate ctxhelm's low-friction local broker contract.

Mitigation:

- import-first precision path
- degraded status instead of hard failure
- optional generator helpers
- no required language toolchain for baseline `prepare_task`

## Pitfall 4: Cloud Policy Drift

Embedding or reranking source code remotely can violate user trust if enabled casually.

Mitigation:

- disabled by default
- explicit repo policy
- provider-specific privacy status in every pack/eval/proof
- redaction and allowed-data-class controls
- no cloud promotion without eval gate and user approval

## Pitfall 5: Optimizing Only RefactoringMiner

RefactoringMiner is useful for Java history and scale, but it may reward exact identifiers and history differently than TS/Python product repos.

Mitigation:

- keep RefactoringMiner as the main large-history regression suite
- add at least one small mixed-language semantic fixture
- report corpus-specific verdicts rather than universal claims
