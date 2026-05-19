# Roadmap: Repo Context Packer

## Overview

This roadmap starts v2.4 Production Semantic & Precision Backends. v2.3 proved that ctxpack can run source-free fixed-corpus evaluation and showed a hard product truth: the current semantic scaffold does not improve RefactoringMiner Recall@10 and increases runtime. v2.4 therefore treats semantic and precision retrieval as measured hypotheses, not default features.

The milestone builds a real local semantic backend, richer precision-enriched semantic documents, structured query construction, provider/reranker policy gates, and hard eval gates for promotion or rollback.

## v2.4 Production Semantic & Precision Backends

## Phases

**Phase Numbering:**

- Integer phases (56, 57, 58, 59, 60): Planned v2.4 work
- Decimal phases (56.1, 57.1): Urgent insertions if needed

- [x] **Phase 56: Production Local Semantic Backend** - Maintainers can build, inspect, and evaluate a real local embedding-backed semantic index without uploading source code.
- [x] **Phase 57: Precision-Enriched Semantic Documents** - Maintainers can enrich semantic and graph retrieval with typed symbol/test/docs context and optional SCIP/LSP precision status.
- [ ] **Phase 58: Query Construction And Hybrid Fusion Controls** - Maintainers can inspect and tune task/commit/error query facets and paired hybrid retrieval variants.
- [ ] **Phase 59: Provider And Reranker Policy Gates** - Maintainers can configure optional cloud/local provider and reranker policies without violating local-first defaults.
- [ ] **Phase 60: Semantic/Precision Evaluation Gates And Release Proof** - Maintainers can prove, block, or roll back semantic/precision defaults using fixed-corpus gates and product proof.

## Phase Details

### Phase 56: Production Local Semantic Backend

**Goal**: Maintainers can build, inspect, and evaluate a real local embedding-backed semantic index without uploading source code.

**Depends on**: v1.4 semantic scaffold, v1.3 storage, v2.3 fixed-corpus eval

**Requirements**: SEM-01, SEM-02, SEM-03, SEM-04

**Success Criteria** (what must be TRUE):

1. A real local embedding provider can embed safe semantic documents and persist source-free vector/provider metadata.
2. `local_hash` remains available only as deterministic test/scaffold behavior and is labeled accordingly.
3. CLI and reports expose provider model id, dimensions, freshness, cache status, privacy status, and degraded/error state.
4. Semantic candidates carry typed provenance and source-free eval features without breaking existing CLI/MCP contracts.

**Plans**: 1 plan

Plans:

- [x] 56-production-local-semantic-backend-01-PLAN.md - Add the real local embedding provider, vector metadata persistence, provider status, and scaffold/backward compatibility boundaries.

### Phase 57: Precision-Enriched Semantic Documents

**Goal**: Maintainers can enrich semantic and graph retrieval with typed symbol/test/docs context and optional SCIP/LSP precision status.

**Depends on**: Phase 56, v1.5 precision bridge, Tree-sitter symbol extraction

**Requirements**: PREC-01, PREC-02, PREC-03, PREC-04

**Success Criteria** (what must be TRUE):

1. Semantic documents include safe path, role, language, symbol, signature, import/export, test, docs/card, and precision facets.
2. SCIP/LSP precision inputs report unavailable/present/stale/failed/partial/degraded status.
3. Existing `prepare-task`, `search`, `related`, and `get-pack` use precision-enriched evidence additively when available.
4. Eval exports, reports, and product proof remain source-free.

**Plans**: 1 plan

Plans:

- [x] 57-precision-enriched-semantic-documents-01-PLAN.md - Build semantic document contracts, precision status reporting, and safe enrichment into retrieval surfaces.

### Phase 58: Query Construction And Hybrid Fusion Controls

**Goal**: Maintainers can inspect and tune task/commit/error query facets and paired hybrid retrieval variants.

**Depends on**: Phases 56-57

**Requirements**: QUERY-01, QUERY-02, QUERY-03, QUERY-04

**Success Criteria** (what must be TRUE):

1. Task, commit, current-diff, explicit-path, symbol, and error-like inputs become structured query facets.
2. CLI/eval output can expose source-free query construction traces.
3. Fusion distinguishes lexical, semantic, precision, graph, history, test, memory, feedback, and active-anchor signals.
4. Fixed-budget variants can compare semantic and precision changes against the same baseline.

**Plans**: 1 plan

Plans:

- [x] 58-query-construction-hybrid-fusion-controls-01-PLAN.md - Add query facet contracts, debug traces, and paired hybrid variant controls.

### Phase 59: Provider And Reranker Policy Gates

**Goal**: Maintainers can configure optional cloud/local provider and reranker policies without violating local-first defaults.

**Depends on**: Phase 58

**Requirements**: PROVIDER-01, PROVIDER-02, PROVIDER-03, PROVIDER-04

**Success Criteria** (what must be TRUE):

1. Cloud embeddings and reranking stay disabled until explicit repo policy enables them.
2. Provider policies record allowed data classes, source-snippet permissions, provider/model/version/dimensions, cost/runtime notes, and rollback targets.
3. Optional reranking can score first-stage candidates without adding noisy MCP tools.
4. CLI/MCP outputs include privacy warnings whenever non-default provider policy affects candidates.

**Plans**: 1 plan

Plans:

- [x] 59-provider-reranker-policy-gates-01-PLAN.md - Add provider/reranker policy contracts, privacy warnings, and local/cloud gated execution controls.

### Phase 60: Semantic/Precision Evaluation Gates And Release Proof

**Goal**: Maintainers can prove, block, or roll back semantic/precision defaults using fixed-corpus gates and product proof.

**Depends on**: Phases 56-59

**Requirements**: GATE-01, GATE-02, GATE-03, GATE-04

**Success Criteria** (what must be TRUE):

1. Fixed-corpus ablations compare default, lexical, local semantic, precision-enriched semantic, semantic-plus-precision, and reranked variants.
2. Reports include Recall@K, MRR@K, test recall, precision proxy, runtime, cache-hit rate, token ROI, failure-family deltas, provider status, and privacy status.
3. Default promotion is blocked unless thresholds beat baseline quality, runtime, token ROI, and privacy gates.
4. Product proof and docs honestly state whether v2.4 produced lift, neutral results, or regressions.

**Plans**: 1 plan

Plans:

- [x] 60-semantic-precision-evaluation-gates-release-proof-01-PLAN.md - Add paired backend ablations, promotion gates, docs, release smoke, and proof-boundary reporting.

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| SEM-01 | Phase 56 |
| SEM-02 | Phase 56 |
| SEM-03 | Phase 56 |
| SEM-04 | Phase 56 |
| PREC-01 | Phase 57 |
| PREC-02 | Phase 57 |
| PREC-03 | Phase 57 |
| PREC-04 | Phase 57 |
| QUERY-01 | Phase 58 |
| QUERY-02 | Phase 58 |
| QUERY-03 | Phase 58 |
| QUERY-04 | Phase 58 |
| PROVIDER-01 | Phase 59 |
| PROVIDER-02 | Phase 59 |
| PROVIDER-03 | Phase 59 |
| PROVIDER-04 | Phase 59 |
| GATE-01 | Phase 60 |
| GATE-02 | Phase 60 |
| GATE-03 | Phase 60 |
| GATE-04 | Phase 60 |

**Coverage:** 20/20 v2.4 requirements mapped. No orphaned requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 56 -> 57 -> 58 -> 59 -> 60

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 56. Production Local Semantic Backend | 1/1 | Complete | 2026-05-19 |
| 57. Precision-Enriched Semantic Documents | 1/1 | Complete | 2026-05-20 |
| 58. Query Construction And Hybrid Fusion Controls | 1/1 | Planned | - |
| 59. Provider And Reranker Policy Gates | 1/1 | Planned | - |
| 60. Semantic/Precision Evaluation Gates And Release Proof | 1/1 | Planned | - |

---
*Roadmap created: 2026-05-19*
