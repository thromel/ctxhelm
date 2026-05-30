# Roadmap: Repo Context Packer

## Overview

This roadmap starts v2.5 Production Retrieval Quality. v2.4 made semantic, precision, provider, and reranker paths source-safe and policy-gated, then the fresh RefactoringMiner proof fixed a semantic fusion regression. The current product gap is quality lift: default and `local_hash` now match each other at Recall@10 `0.6355`, but lexical baseline remains ahead at `0.6665`.

v2.5 therefore focuses on measured retrieval quality, not more surface area. The milestone must prove whether production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline on real repositories while staying local-first and source-safe.

## v2.5 Production Retrieval Quality

## Phases

**Phase Numbering:**

- Integer phases (61, 62, 63, 64, 65): Planned v2.5 work
- Decimal phases (61.1, 62.1): Urgent insertions if needed

- [x] **Phase 61: Multi-Repo Quality Baselines** - Maintainers can run source-free paired baselines across RefactoringMiner and a second real repository with stable comparison artifacts.
- [x] **Phase 62: Production Local Embedding Quality** - Maintainers can evaluate production local embeddings against lexical/default baselines with bounded local cache and provider metadata.
- [x] **Phase 63: Reranker And Fusion Promotion** - Maintainers can compare reranker/fusion variants under promotion gates that protect anchors and exact evidence.
- [ ] **Phase 64: Gap-Family Retrieval Improvements** - Maintainers can convert repeated gap families into targeted retrieval fixes with before/after proof.
- [ ] **Phase 65: v2.5 Product Proof And Release Gate** - Maintainers can ship or hold v2.5 variants using multi-repo proof, docs, and release gates.

## Phase Details

### Phase 61: Multi-Repo Quality Baselines

**Goal**: Maintainers can run source-free paired baselines across RefactoringMiner and a second real repository with stable comparison artifacts.

**Depends on**: v2.3 fixed-corpus eval, v2.4 semantic proof

**Requirements**: BASE-01, BASE-02, BASE-03, BASE-04

**Success Criteria**:

1. Baseline command/report covers at least RefactoringMiner and one second real repo.
2. Reports include stable corpus identity, revision range, provider status, runtime, cache, privacy, and named gap families.
3. Default, lexical, graph, semantic, reranked, and learned-policy variants can be compared deterministically.
4. Reports remain source-free.

**Plans**: 1 plan

Plans:

- [x] 61-multi-repo-quality-baselines-01-PLAN.md - Build the multi-repo baseline manifest/report path and prove it on real repos.

### Phase 62: Production Local Embedding Quality

**Goal**: Maintainers can evaluate production local embeddings against lexical/default baselines with bounded local cache and provider metadata.

**Depends on**: Phase 61, v2.4 local provider policy

**Requirements**: EMBED-01, EMBED-02, EMBED-03, EMBED-04

**Success Criteria**:

1. Production local embedding backend runs from CLI/eval without cloud transfer.
2. Cache behavior is bounded, ignored by inventory, and visible in status reports.
3. Quality is measured against lexical/default before promotion.
4. `local_hash` remains scaffold-labeled.

**Plans**: 1 plan

Plans:

- [x] 62-production-local-embedding-quality-01-PLAN.md - Harden local embedding quality, cache behavior, and provider evidence.

### Phase 63: Reranker And Fusion Promotion

**Goal**: Maintainers can compare reranker/fusion variants under promotion gates that protect anchors and exact evidence.

**Depends on**: Phases 61-62

**Requirements**: RANK-01, RANK-02, RANK-03, RANK-04

**Success Criteria**:

1. Reranker/fusion variants are source-safe and do not expand MCP tool surface.
2. Anchors, current diff, exact lexical, and strong symbols are protected.
3. Promotion gates compare quality, runtime, token ROI, and privacy.
4. Named regressions block promotion.

**Plans**: 1 plan

Plans:

- [x] 63-reranker-and-fusion-promotion-01-PLAN.md - Implement and evaluate promotion-safe reranking/fusion variants.

### Phase 64: Gap-Family Retrieval Improvements

**Goal**: Maintainers can convert repeated gap families into targeted retrieval fixes with before/after proof.

**Depends on**: Phase 61

**Requirements**: GAP-01, GAP-02, GAP-03, GAP-04

**Success Criteria**:

1. Gap families are grouped into actionable tasks.
2. At least one high-impact RefactoringMiner gap family improves with measured before/after proof.
3. Test recommendation quality is evaluated separately from target-file recall.
4. Graph expansion stays budgeted and seed-safe.

**Plans**: 1 plan

Plans:

- [ ] 64-gap-family-retrieval-improvements-01-PLAN.md - Fix one or more measured gap families and prove improvement.

### Phase 65: v2.5 Product Proof And Release Gate

**Goal**: Maintainers can ship or hold v2.5 variants using multi-repo proof, docs, and release gates.

**Depends on**: Phases 61-64

**Requirements**: PROOF-01, PROOF-02, PROOF-03, PROOF-04

**Success Criteria**:

1. Product proof states beat/match/trail status per corpus and variant.
2. Release gate blocks neutral, mixed, unsafe, or too-expensive defaults.
3. Docs tell users which retrieval mode to use today.
4. Workspace validation and source-free E2E proof pass.

**Plans**: 1 plan

Plans:

- [ ] 65-v25-product-proof-release-gate-01-PLAN.md - Finalize measured proof, docs, and release gates.

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| BASE-01 | Phase 61 |
| BASE-02 | Phase 61 |
| BASE-03 | Phase 61 |
| BASE-04 | Phase 61 |
| EMBED-01 | Phase 62 |
| EMBED-02 | Phase 62 |
| EMBED-03 | Phase 62 |
| EMBED-04 | Phase 62 |
| RANK-01 | Phase 63 |
| RANK-02 | Phase 63 |
| RANK-03 | Phase 63 |
| RANK-04 | Phase 63 |
| GAP-01 | Phase 64 |
| GAP-02 | Phase 64 |
| GAP-03 | Phase 64 |
| GAP-04 | Phase 64 |
| PROOF-01 | Phase 65 |
| PROOF-02 | Phase 65 |
| PROOF-03 | Phase 65 |
| PROOF-04 | Phase 65 |

**Coverage:** 20/20 v2.5 requirements mapped. No orphaned requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 61 -> 62 -> 63 -> 64 -> 65

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 61. Multi-Repo Quality Baselines | 1/1 | Complete | 2026-05-22 |
| 62. Production Local Embedding Quality | 1/1 | Complete | 2026-05-30 |
| 63. Reranker And Fusion Promotion | 0/1 | Planned | |
| 64. Gap-Family Retrieval Improvements | 0/1 | Planned | |
| 65. v2.5 Product Proof And Release Gate | 0/1 | Planned | |

---
*Roadmap created: 2026-05-22*
