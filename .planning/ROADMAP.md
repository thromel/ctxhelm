# Roadmap: Repo Context Packer

## Overview

This roadmap tracks v2.5 Production Retrieval Quality and its immediate production-readiness follow-ups. v2.4 made semantic, precision, provider, and reranker paths source-safe and policy-gated, then the fresh RefactoringMiner proof fixed a semantic fusion regression. The current fixed two-repo product proof promotes default local retrieval under a channel-aware gate: non-test context recall beats lexical on both corpora, while validation-test recall is measured separately through `recommended_tests`.

v2.5 therefore focuses on measured retrieval quality, not more surface area. The milestone must prove whether production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline on real repositories while staying local-first and source-safe. Phase 66 fixed the false zero-test-recall signal by measuring `recommended_tests` as its own validation channel. Phase 67 fixed the denominator for historical retrieval metrics by separating all safe changed files from parent-snapshot `retrievalTargetFiles`. Phase 69 promoted default local retrieval under the channel-aware proof, Phase 70 refreshed real-client MCP evidence for Codex CLI and Claude Code, Phase 71 reduced archive-artifact retrieval noise in ctxpack's own history, Phase 72 broadened repeated-lift validation while improving validation-test recall seeding, Phase 73 pinned a broader optional fixed-corpus probe, Phase 76 split partial-snapshot history into validation-only mode for historical eval, Phase 77 added broad validation-command coverage for multi-area smoke/eval tasks, Phase 78 made the broader proof gate lexical-ceiling aware, Phase 79 added protected target floors, Phase 80 fixed symbol-floor duplicate accounting, Phase 81 made warm-cache runtime proof trustworthy, Phase 82 made warm-cache runtime enforceable, Phase 83 made context-vs-all-file divergence machine-checkable, Phase 84 added broad-scope task accounting plus scoped dependency source floors, Phase 85 added source-free context-area hints for broad prepare-task plans and packs, Phase 86 added bounded Python package re-export graph coverage, and Phase 87 fixed validation-covered test gap accounting.

## v2.5 Production Retrieval Quality

## Phases

**Phase Numbering:**

- Integer phases (61, 62, 63, 64, 65): Planned v2.5 work
- Phases 66-87: Production-readiness follow-ups from the original blocked proof and the channel-aware promotion path
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
- [x] **Phase 74: Protected Evidence Diagnostics** - Maintainers can separate protected retrieval-target misses from non-target exact/symbol pressure.
- [x] **Phase 75: Parent-Bounded History And Test Reserve** - Maintainers can preserve source-free parent-bounded history and reserve co-changed validation tests.
- [x] **Phase 76: Parent-Bounded Validation History** - Maintainers can use parent-bounded history for historical eval validation tests without perturbing non-test target ranking from partial snapshots.
- [x] **Phase 77: Validation Command Coverage** - Maintainers can represent broad multi-area validation tasks with suite-level fallback commands and effective validation recall.
- [x] **Phase 78: Ceiling-Aware Broader Gate** - Maintainers can promote broader proof when a corpus reaches a safe lexical ceiling instead of falsely requiring impossible recall lift.
- [x] **Phase 79: Protected Target Floors** - Maintainers can reduce protected retrieval-target misses by reserving bounded source/config/governance evidence and deferring archive artifacts.
- [x] **Phase 80: Unique Symbol Floor Accounting** - Maintainers can keep symbol-only source evidence inside budget when duplicate already-selected symbol files appear first.
- [x] **Phase 81: Warm Cache Latency Proof** - Maintainers can prove cache-hit eval runtime with source-free cold/warm product proof artifacts.
- [x] **Phase 82: Warm Cache Release Gate** - Maintainers can block product-proof promotion when cached runtime evidence is stale or too slow.
- [x] **Phase 83: Context Divergence Accounting** - Maintainers can distinguish useful context-channel lift from all-file lexical deficits caused by validation targets.
- [x] **Phase 84: Broad Scope Dependency Floors** - Maintainers can identify broad multi-area tasks and preserve bounded dependency source evidence for them.
- [x] **Phase 85: Broad Context Areas** - Agents can inspect source-free area hints for broad multi-area tasks without displacing target files or validation channels.
- [x] **Phase 86: Python Package Re-Export Graph Coverage** - Python package `__init__.py` re-exports can contribute bounded graph candidates without enabling general depth-2 expansion.
- [x] **Phase 87: Validation Gap Accounting** - Proof reports stop classifying validation-covered tests as unresolved retrieval gaps, and Java class-selector commands count toward validation coverage.

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

### Phase 74: Protected Evidence Diagnostics

**Goal**: Maintainers can tell whether protected-evidence misses are actual
retrievable target misses or non-target exact/symbol pressure.

**Depends on**: Phase 73

**Requirements**: PROOF-01, PROOF-02, RANK-02

**Success Criteria**:

1. Protected evidence files include source-free retrieval-target status and file role.
2. Protected evidence summaries report total, retrieval-target, and non-target miss counts.
3. Product-proof corpus verdicts expose protected retrieval-target miss-rate.
4. Required and broader proofs document the remaining target misses without changing retrieval ranking.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase74-protected-evidence-diagnostics.md`
- [x] `.ctxpack/e2e/phase74-protected-evidence-diagnostics-proof.json`
- [x] `.ctxpack/e2e/phase74-broader-protected-evidence-diagnostics-proof.json`

### Phase 75: Parent-Bounded History And Test Reserve

**Goal**: Historical eval snapshots preserve bounded prior co-change history,
and validation-test selection protects tests that co-change with target files.

**Depends on**: Phase 74

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. Parent snapshots can use source-free commit/path history without a full Git checkout.
2. The eval-history sidecar is excluded from inventory and context selection.
3. Co-changed validation tests get a reserved selection pass before generic matches.
4. Required proof still promotes and broader proof records whether VeriSchema improves.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase75-parent-history-test-reserve.md`
- [x] `.ctxpack/e2e/phase75-parent-history-test-reserve-proof.json`
- [x] `.ctxpack/e2e/phase75-broader-parent-history-test-reserve-proof.json`

### Phase 76: Parent-Bounded Validation History

**Goal**: Historical eval snapshots use source-free parent-bounded history for
validation-test discovery without using partial snapshot history as a general
non-test target ranking signal.

**Depends on**: Phase 75

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. Historical eval distinguishes full history from validation-only history.
2. Parent snapshots use sidecar history for related-test discovery and command generation.
3. Partial snapshot history does not perturb non-test target context ranking.
4. Required proof still promotes and broader proof records VeriSchema validation-test movement.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase76-parent-bounded-validation-history.md`
- [x] `.ctxpack/e2e/phase76-parent-bounded-validation-history-proof.json`
- [x] `.ctxpack/e2e/phase76-broader-parent-bounded-validation-history-proof.json`

### Phase 77: Validation Command Coverage

**Goal**: Broad multi-area smoke/eval tasks can recommend suite-level fallback
commands and prove effective validation coverage without hiding raw top-10 test
recall.

**Depends on**: Phase 76

**Requirements**: PROOF-01, PROOF-02, GAP-03

**Success Criteria**:

1. Broad validation tasks add fallback commands after targeted test commands.
2. Historical eval reports raw Test Recall@10 and command-backed effective validation recall separately.
3. Product proof uses effective validation recall for validation floors while preserving raw test recall diagnostics.
4. Required proof still promotes and broader proof records remaining corpus blockers.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase77-validation-command-coverage.md`
- [x] `.ctxpack/e2e/phase77-validation-command-coverage-proof.json`
- [x] `.ctxpack/e2e/phase77-broader-validation-command-coverage-proof.json`

### Phase 78: Ceiling-Aware Broader Gate

**Goal**: Safe perfect lexical-ceiling matches should not block broader
production-readiness proof when validation is healthy and protected target
misses are zero.

**Depends on**: Phase 77

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Ordinary non-ceiling `match` verdicts still block promotion.
2. Perfect context-channel ceiling matches with zero protected target misses can promote.
3. Product-proof checker accepts only `beat` or safe perfect-ceiling `match` verdicts.
4. Broader fixed-corpus proof promotes while preserving protected-miss diagnostics.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase78-ceiling-aware-broader-gate.md`
- [x] `.ctxpack/e2e/phase78-ceiling-aware-broader-proof.json`

### Phase 79: Protected Target Floors

**Goal**: Protected source/config/governance evidence should survive standard
budget selection more reliably, while archive artifacts remain available as
fallback context.

**Depends on**: Phase 78

**Requirements**: PROOF-01, PROOF-02, RANK-02

**Success Criteria**:

1. Source lexical and source symbol candidates receive bounded floors under no-active-context tasks.
2. Exact config candidates and agent setup governance docs receive bounded floors.
3. Archive artifacts from `.planning/e2e`, `.planning/phases`, and `.planning/milestones` are deferred during fill.
4. Required and broader proofs still promote and record protected target miss movement.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase79-protected-target-floors.md`
- [x] `.ctxpack/e2e/phase79-protected-target-floors-proof.json`
- [x] `.ctxpack/e2e/phase79-broader-protected-target-floors-proof.json`

### Phase 80: Unique Symbol Floor Accounting

**Goal**: Symbol floor limits should count unique newly selected files, not
duplicate attempts for files already selected by lexical floors.

**Depends on**: Phase 79

**Requirements**: PROOF-01, PROOF-02, RANK-02

**Success Criteria**:

1. Source-symbol floor runs before governance/doc fill for no-active-context tasks.
2. Source-symbol and general symbol floors count only newly selected files against their limits.
3. Required and broader proofs promote with protected retrieval-target miss-rate `0.0` in measured corpora.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase80-unique-symbol-floor.md`
- [x] `.ctxpack/e2e/phase80-unique-symbol-floor-proof.json`
- [x] `.ctxpack/e2e/phase80-broader-unique-symbol-floor-proof.json`

### Phase 81: Warm Cache Latency Proof

**Goal**: Cached historical eval reports should report warm lookup runtime,
not stale cold-run timings.

**Depends on**: Phase 80

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Cache-hit reports show warm lookup runtime with zero commit-loop, ranking,
   pack/compiler, git sample, and slow-commit timings.
2. Cold and warm product-proof artifacts promote on the four-repo fixed corpus.
3. Warm proof records cache hits and no cache misses for every measured corpus.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase81-warm-cache-latency.md`
- [x] `.planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json`
- [x] `.ctxpack/e2e/phase81-warm-cache-cold-proof.json`
- [x] `.ctxpack/e2e/phase81-warm-cache-warm-proof.json`

### Phase 82: Warm Cache Release Gate

**Goal**: Warm-cache product proof should be an enforceable release gate, not
only a diagnostic artifact.

**Depends on**: Phase 81

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Cache-hit product proofs block if warm runtime carries stale cold timings.
2. Cache-hit product proofs block if warm lookup runtime exceeds `1000ms`.
3. Clean cold/warm proof replay still promotes and records warm-cache notes.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase82-warm-cache-gate.md`
- [x] `.ctxpack/e2e/phase82-warm-cache-gate-cold-proof.json`
- [x] `.ctxpack/e2e/phase82-warm-cache-gate-warm-proof.json`

### Phase 83: Context Divergence Accounting

**Goal**: Context-vs-all-file corpus divergence should be machine-checkable,
not only explained in prose notes.

**Depends on**: Phase 69, Phase 77, Phase 82

**Requirements**: PROOF-01, PROOF-02

**Success Criteria**:

1. Product-proof corpus verdicts expose context-vs-all-file deltas for ctxpack
   and lexical.
2. Product-proof promotion blocks unexplained all-file lexical deficits.
3. The source-free product-proof checker fails if divergence fields are missing
   or if an all-file deficit is not explained.
4. The broader four-repo proof still promotes with explained RefactoringMiner
   and ReAgent all-file deficits.

**Evidence**:

- [x] `.planning/e2e/2026-05-30-phase83-context-divergence-accounting.md`
- [x] `.ctxpack/e2e/phase83-context-divergence-proof.json`

### Phase 84: Broad Scope Dependency Floors

**Goal**: Broad workflow/eval/lint tasks should be visible in eval output and
should not lose dependency source evidence to unrelated context when the task
spans many files.

**Depends on**: Phase 77, Phase 80, Phase 83

**Requirements**: GAP-02, GAP-04, PROOF-01

**Success Criteria**:

1. Prepare-task emits `multi_area_task` diagnostics for broad workflow/eval/lint
   prompts.
2. Historical eval JSON includes `broadScopeCommitCount` and per-commit
   `broadScopeTask`.
3. Dependency source floors activate only for broad-scope tasks, avoiding the
   RefactoringMiner regression seen with an unconditional floor.
4. The broader four-repo proof still promotes and improves VeriSchema source
   recall.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase84-broad-scope-dependency-floors.md`
- [x] `.ctxpack/e2e/phase84-broad-scope-dependency-proof.json`

### Phase 85: Broad Context Areas

**Goal**: Broad multi-area prepare-task plans and packs should expose adjacent
repository areas as source-free guidance without changing protected target-file,
test, or validation budgets.

**Depends on**: Phase 84

**Requirements**: GAP-02, GAP-04, PROOF-01

**Success Criteria**:

1. `ContextPlan` exposes typed additive `contextAreas`.
2. `prepare-task` populates `contextAreas` only for broad multi-area tasks.
3. Packs render context areas as inspection hints after risk flags.
4. Focused tests cover the public JSON shape, source-free contract, and
   multi-area diagnostics.
5. Proof documents that broad fixed-corpus quality metrics are unchanged and
   the warm-cache proof still promotes.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase85-broad-context-areas.md`
- [x] `.ctxpack/e2e/phase85-context-areas-warm-proof.json`

### Phase 86: Python Package Re-Export Graph Coverage

**Goal**: Python package re-exports should appear in graph retrieval so
`from package import Symbol` tasks can surface concrete module files without
general recursive graph expansion.

**Depends on**: Phase 84, Phase 85

**Requirements**: GAP-02, GAP-04

**Success Criteria**:

1. Python import extraction records imported submodule paths for
   `from module import member`.
2. Dependency resolution recognizes `package/__init__.py`.
3. Related dependency expansion adds bounded `python_reexport` edges from an
   anchor through package `__init__.py` to re-exported modules.
4. Focused dependency tests cover absolute submodule imports, relative
   `from . import module`, and package re-export expansion.
5. Broader proof documents no protected-target regression even if top-10 recall
   remains flat.

**Evidence**:

- [x] `.planning/e2e/2026-05-31-phase86-python-package-reexports.md`

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
| PROOF-01 | Phase 74 |
| PROOF-02 | Phase 74 |
| RANK-02 | Phase 74 |
| PROOF-01 | Phase 75 |
| PROOF-02 | Phase 75 |
| GAP-03 | Phase 75 |
| PROOF-01 | Phase 76 |
| PROOF-02 | Phase 76 |
| GAP-03 | Phase 76 |
| PROOF-01 | Phase 77 |
| PROOF-02 | Phase 77 |
| GAP-03 | Phase 77 |
| PROOF-01 | Phase 78 |
| PROOF-02 | Phase 78 |
| PROOF-01 | Phase 79 |
| PROOF-02 | Phase 79 |
| RANK-02 | Phase 79 |
| PROOF-01 | Phase 80 |
| PROOF-02 | Phase 80 |
| RANK-02 | Phase 80 |
| PROOF-01 | Phase 81 |
| PROOF-02 | Phase 81 |
| PROOF-01 | Phase 82 |
| PROOF-02 | Phase 82 |
| PROOF-01 | Phase 83 |
| PROOF-02 | Phase 83 |
| GAP-02 | Phase 84 |
| GAP-04 | Phase 84 |
| PROOF-01 | Phase 84 |
| GAP-02 | Phase 85 |
| GAP-04 | Phase 85 |
| PROOF-01 | Phase 85 |
| GAP-02 | Phase 86 |
| GAP-04 | Phase 86 |

**Coverage:** 20/20 v2.5 requirements mapped, with Phases 66-86 as measured follow-ups for proof/eval correctness gaps, real-client evidence, archive-noise reduction, broader validation, fixed-corpus reproducibility, protected-evidence diagnostics, parent-bounded history/test reservation, validation-only historical eval history, validation-command coverage, lexical-ceiling proof semantics, protected target floors, symbol-floor accounting, warm-cache runtime proof, warm-cache release gating, context-vs-all-file divergence accounting, broad-scope dependency source floors, broad context-area hints, and Python package re-export graph coverage. No orphaned v2.5 requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 61 -> 62 -> 63 -> 64 -> 65 -> 66 -> 67 -> 69 -> 70 -> 71 -> 72 -> 73 -> 74 -> 75 -> 76 -> 77 -> 78 -> 79 -> 80 -> 81 -> 82 -> 83 -> 84 -> 85 -> 86

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
| 74. Protected Evidence Diagnostics | Evidence artifact | Complete | 2026-05-30 |
| 75. Parent-Bounded History And Test Reserve | Evidence artifact | Complete | 2026-05-30 |
| 76. Parent-Bounded Validation History | Evidence artifact | Complete | 2026-05-30 |
| 77. Validation Command Coverage | Evidence artifact | Complete | 2026-05-30 |
| 78. Ceiling-Aware Broader Gate | Evidence artifact | Complete | 2026-05-30 |
| 79. Protected Target Floors | Evidence artifact | Complete | 2026-05-30 |
| 80. Unique Symbol Floor Accounting | Evidence artifact | Complete | 2026-05-30 |
| 81. Warm Cache Latency Proof | Evidence artifact | Complete | 2026-05-30 |
| 82. Warm Cache Release Gate | Evidence artifact | Complete | 2026-05-30 |
| 83. Context Divergence Accounting | Evidence artifact | Complete | 2026-05-30 |
| 84. Broad Scope Dependency Floors | Evidence artifact | Complete | 2026-05-31 |
| 85. Broad Context Areas | Evidence artifact | Complete | 2026-05-31 |
| 86. Python Package Re-Export Graph Coverage | Evidence artifact | Complete | 2026-05-31 |

---
*Roadmap created: 2026-05-22*
