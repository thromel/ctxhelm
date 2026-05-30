# Roadmap: Repo Context Packer

## Overview

This roadmap tracks v2.5 Production Retrieval Quality and its immediate production-readiness follow-ups. v2.4 made semantic, precision, provider, and reranker paths source-safe and policy-gated, then the fresh RefactoringMiner proof fixed a semantic fusion regression. The current fixed two-repo product proof promotes default local retrieval under a channel-aware gate: non-test context recall beats lexical on both corpora, while validation-test recall is measured separately through `recommended_tests`.

v2.5 therefore focuses on measured retrieval quality, not more surface area. The milestone must prove whether production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline on real repositories while staying local-first and source-safe. Phase 66 fixed the false zero-test-recall signal by measuring `recommended_tests` as its own validation channel. Phase 67 fixed the denominator for historical retrieval metrics by separating all safe changed files from parent-snapshot `retrievalTargetFiles`. Phase 69 promoted default local retrieval under the channel-aware proof, Phase 70 refreshed real-client MCP evidence for Codex CLI and Claude Code, Phase 71 reduced archive-artifact retrieval noise in ctxpack's own history, Phase 72 broadened repeated-lift validation while improving validation-test recall seeding, and Phase 73 pinned a broader optional fixed-corpus probe.

## v2.5 Production Retrieval Quality

## Phases

**Phase Numbering:**

- Integer phases (61, 62, 63, 64, 65): Planned v2.5 work
- Phases 66-73: Production-readiness follow-ups from the original blocked proof and the channel-aware promotion path
- Decimal phases (61.1, 62.1): Urgent insertions if needed

- [x] **Phase 61: Multi-Repo Quality Baselines** - Maintainers can run source-free paired baselines across RefactoringMiner and a second real repository with stable comparison artifacts.
- [x] **Phase 62: Production Local Embedding Quality** - Maintainers can evaluate production local embeddings against lexical/default baselines with bounded local cache and provider metadata.
- [x] **Phase 63: Reranker And Fusion Promotion** - Maintainers can compare reranker/fusion variants under promotion gates that protect anchors and exact evidence.
- [x] **Phase 64: Gap-Family Retrieval Improvements** - Maintainers can convert repeated gap families into targeted retrieval fixes with before/after proof.
- [x] **Phase 65: v2.5 Product Proof And Release Gate** - Maintainers can ship or hold v2.5 variants using multi-repo proof, docs, and release gates.
- [x] **Phase 66: Test Recall Evaluation Channel** - Maintainers can measure validation-test recall through the dedicated `recommended_tests` output without degrading target-file recall.
- [x] **Phase 67: Retrievable Target Eval Denominator** - Maintainers can distinguish all safe changed files from files that existed in the parent snapshot and could be retrieved as context.
- [x] **Phase 69: Channel-Aware Product Proof Gate** - Maintainers can promote default local retrieval when context recall beats lexical while validation-test recall is proven separately.
- [x] **Phase 70: Real-Client MCP Proof Refresh** - Maintainers can verify Codex CLI and Claude Code still invoke `prepare_task` and `get_pack` through actual MCP client paths after promotion.
- [x] **Phase 71: Archive Artifact Dampening** - Maintainers can reduce ctxpack planning-archive retrieval noise without excluding archived evidence from search.
- [x] **Phase 72: Broader Repeated-Lift Validation** - Maintainers can probe more local repositories, improve validation-test recall seeding, and keep broader promotion gaps explicit.
- [x] **Phase 73: Broader Fixed-Corpus Fixture** - Maintainers can rerun the broader probe from a pinned committed config instead of a temporary manifest.

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

- [x] 64-gap-family-retrieval-improvements-01-PLAN.md - Fix one or more measured gap families and prove improvement.

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

- [x] 65-v25-product-proof-release-gate-01-PLAN.md - Finalize measured proof, docs, and release gates.

### Phase 66: Test Recall Evaluation Channel

**Goal**: Maintainers can measure validation-test recall through the dedicated related-tests channel instead of treating tests as absent when the target-file ranking is full.

**Depends on**: Phase 65

**Requirements**: GAP-03, PROOF-01

**Success Criteria**:

1. Test Recall@10 is measured from `recommended_tests`.
2. Target-file ranking behavior is not degraded by forced test-slot reservation.
3. Related-test ordering preserves raw score differences hidden by capped public confidence.
4. The product proof still blocks default promotion until every corpus beats lexical.

**Plans**: 1 plan

Plans:

- [x] 66-test-recall-eval-channel-01-PLAN.md - Correct the validation-test evaluation channel and prove the result.

### Phase 67: Retrievable Target Eval Denominator

**Goal**: Maintainers can evaluate retrieval quality against files that existed in the parent snapshot while preserving the full source-free patch audit list.

**Depends on**: Phase 66

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Historical eval emits `retrievalTargetFiles`.
2. `safeChangedFiles` continues to show the full safe patch surface.
3. Recall, MRR, token ROI, role recall, missing-file, and gap metrics use retrievable parent-snapshot targets.
4. Product proof remains blocked unless every corpus beats lexical.

**Plans**: 1 plan

Plans:

- [x] 67-retrievable-target-eval-denominator-01-PLAN.md - Make retrieval metrics use parent-snapshot retrievable targets.

### Phase 69: Channel-Aware Product Proof Gate

**Goal**: Maintainers can promote default local retrieval when context recall beats lexical and validation-test recall is proven through the dedicated test channel.

**Depends on**: Phase 67

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. Product proof separates context Recall@10 from validation-test Recall@10.
2. Release gate promotes only when required corpora beat lexical on context recall and meet the test-recall floor.
3. Proof notes preserve all-file recall transparency without treating tests as both context targets and validation commands.
4. Source-free JSON proof reports `releaseGate.decision = "promote"`.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase69-channel-aware-product-proof.md`
- [x] `.ctxpack/e2e/phase69-channel-scoped-governance-proof.json`

### Phase 70: Real-Client MCP Proof Refresh

**Goal**: Maintainers can verify that Codex CLI and Claude Code still invoke ctxpack through actual MCP client paths after the Phase 69 promotion.

**Depends on**: Phase 69

**Requirements**: AGENT-01 follow-up evidence

**Success Criteria**:

1. Codex CLI real-client wrapper passes deterministic protocol proof first.
2. Claude Code real-client wrapper passes deterministic protocol proof first.
3. Both wrappers record server-side `prepare_task` and `get_pack` evidence with an explicit repo path.
4. Docs keep Cursor/OpenCode real-client tool-call proof out of scope until machine-checkable client proof exists.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase70-real-client-mcp-proof.md`

### Phase 71: Archive Artifact Dampening

**Goal**: Maintainers can reduce ctxpack planning-archive retrieval noise without excluding archived evidence from search.

**Depends on**: Phase 69

**Requirements**: GAP-01, GAP-02, RANK-02

**Success Criteria**:

1. `.planning/milestones/**` and `.planning/e2e/**/*.json` stay searchable but no longer dominate generic lexical retrieval.
2. Symbol budget reserve activates only when archive lexical artifacts are present.
3. The fixed two-repo proof still promotes default local retrieval.
4. ctxpack protected evidence miss-rate improves on the current-history proof without changing RefactoringMiner.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase71-archive-artifact-dampening.md`
- [x] `.ctxpack/e2e/phase71-archive-artifact-dampening-proof.json`

### Phase 72: Broader Repeated-Lift Validation

**Goal**: Maintainers can probe more local repositories, improve validation-test recall seeding, and keep broader promotion gaps explicit.

**Depends on**: Phase 69, Phase 71

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. The required fixed two-repo proof still promotes after test-selection changes.
2. Related-test selection can return up to 10 tests to align with Test Recall@10.
3. Related-test discovery uses co-changed and dependency-neighbor source files as additional seeds.
4. Broader probe results are documented honestly when they block promotion.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase72-broader-repeated-lift-validation.md`

### Phase 73: Broader Fixed-Corpus Fixture

**Goal**: Maintainers can rerun the broader probe from a pinned committed config instead of a temporary manifest.

**Depends on**: Phase 72

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. The broader probe config is committed under `.planning/e2e`.
2. External repository paths are relative to the config file.
3. Repository heads are pinned so ctxpack development commits do not silently change the probe.
4. Docs state that the broader fixture is optional and currently blocks broader promotion.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json`
- [x] `.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-fixture.md`

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
| GAP-03 | Phase 66 |
| PROOF-01 | Phase 66 |
| PROOF-01 | Phase 67 |
| PROOF-02 | Phase 67 |
| PROOF-01 | Phase 69 |
| PROOF-02 | Phase 69 |
| GAP-03 | Phase 69 |
| AGENT-01 | Phase 70 |
| GAP-01 | Phase 71 |
| GAP-02 | Phase 71 |
| RANK-02 | Phase 71 |
| PROOF-01 | Phase 72 |
| PROOF-02 | Phase 72 |
| GAP-03 | Phase 72 |
| PROOF-01 | Phase 73 |
| PROOF-02 | Phase 73 |

**Coverage:** 20/20 v2.5 requirements mapped, with Phases 66-73 as measured follow-ups for proof/eval correctness gaps, real-client evidence, archive-noise reduction, broader validation, and fixed-corpus reproducibility. No orphaned v2.5 requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 61 -> 62 -> 63 -> 64 -> 65 -> 66 -> 67 -> 69 -> 70 -> 71 -> 72 -> 73

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 61. Multi-Repo Quality Baselines | 1/1 | Complete | 2026-05-22 |
| 62. Production Local Embedding Quality | 1/1 | Complete | 2026-05-30 |
| 63. Reranker And Fusion Promotion | 1/1 | Complete | 2026-05-30 |
| 64. Gap-Family Retrieval Improvements | 1/1 | Complete | 2026-05-30 |
| 65. v2.5 Product Proof And Release Gate | 1/1 | Complete | 2026-05-30 |
| 66. Test Recall Evaluation Channel | 1/1 | Complete | 2026-05-30 |
| 67. Retrievable Target Eval Denominator | 1/1 | Complete | 2026-05-30 |
| 69. Channel-Aware Product Proof Gate | Evidence artifact | Complete | 2026-05-30 |
| 70. Real-Client MCP Proof Refresh | Evidence artifact | Complete | 2026-05-30 |
| 71. Archive Artifact Dampening | Evidence artifact | Complete | 2026-05-30 |
| 72. Broader Repeated-Lift Validation | Evidence artifact | Complete | 2026-05-30 |
| 73. Broader Fixed-Corpus Fixture | Evidence artifact | Complete | 2026-05-30 |

---
*Roadmap created: 2026-05-22*
