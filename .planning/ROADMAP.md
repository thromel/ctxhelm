# Roadmap: Repo Context Packer

## Overview

This roadmap starts v2.3 Evaluation Lab & Learned Retrieval Policy. v2.2 completed release and distribution hardening; v2.3 now makes the product's retrieval-quality claims repeatable, fast enough to iterate on, and strong enough to support learned retrieval policy without storing source text.

The milestone is intentionally evaluation-first. Production semantic backends, cloud rerankers, SCIP automation, and deeper agent integrations remain later milestones until v2.3 can prove which signals actually improve file/test/context selection.

## v2.3 Evaluation Lab & Learned Retrieval Policy

## Phases

**Phase Numbering:**
- Integer phases (50, 51, 52, 53, 54, 55): Planned v2.3 work
- Decimal phases (50.1, 51.1): Urgent insertions if needed

- [x] **Phase 50: Fixed Benchmark Corpus & RefactoringMiner Regression Suite** - Maintainers can define fixed source-free benchmark corpora and lock the first large-history regression suite.
- [x] **Phase 51: Historical Eval Cache & Parallel Runner** - Maintainers can run large-history evals faster through cache reuse, parallel samples, and runtime diagnostics.
- [ ] **Phase 52: Source-Free Candidate Feature Export** - Maintainers can export privacy-safe candidate feature rows for learning, diagnostics, and paired analysis.
- [ ] **Phase 53: Paired Baseline & Ablation Analysis** - Maintainers can compare ctxpack against lexical, no-context, and signal-ablation baselines with honest verdicts.
- [ ] **Phase 54: Offline Learned Retrieval Policy Experiment** - Maintainers can generate, compare, apply, disable, and roll back non-default learned policy profiles.
- [ ] **Phase 55: Product Proof Gates & v2.3 Release Integration** - Maintainers can include bounded v2.3 eval proof in docs and release gates without requiring external repos by default.

## Phase Details

### Phase 50: Fixed Benchmark Corpus & RefactoringMiner Regression Suite

**Goal**: Maintainers can define fixed source-free benchmark corpora and lock the first large-history regression suite.

**Depends on**: v1.2 benchmark/proof reports, v2.2 release proof bundle, RefactoringMiner E2E evidence

**Requirements**: CORPUS-01, CORPUS-02, CORPUS-03, CORPUS-04

**Success Criteria** (what must be TRUE):
1. Corpus manifests record repo paths, revisions, commit ranges, budgets, task types, target agents, and privacy labels.
2. RefactoringMiner 20-commit regression suite records current Recall@10, lexical baseline, runtime, and gap-family baselines.
3. Additional repos can be added through the same manifest format without storing source, prompts, snippets, or private issue descriptions.
4. Benchmark reports contain reproducibility metadata and source-free privacy status.

**Plans**: 1 plan

Plans:
- [x] 50-fixed-benchmark-corpus-refactoringminer-regression-suite-01-PLAN.md - Build corpus manifests, lock RefactoringMiner baseline metadata, and document reproducible source-free benchmark setup.

### Phase 51: Historical Eval Cache & Parallel Runner

**Goal**: Maintainers can run large-history evals faster through cache reuse, parallel samples, and runtime diagnostics.

**Depends on**: Phase 50

**Requirements**: SPEED-01, SPEED-02, SPEED-03, SPEED-04

**Success Criteria** (what must be TRUE):
1. Historical eval reuses warm parent snapshots, inventories, indexes, and candidate metadata when inputs are unchanged.
2. Commit samples run in parallel while final output remains deterministic and source-free.
3. Reports expose total time, per-commit time, cache hits, slow commits, git diff cost, ranking cost, and pack/compiler cost.
4. Stored run comparison avoids recomputing unchanged benchmark ranges unless explicitly forced.

**Plans**: 1 plan

Plans:
- [x] 51-historical-eval-cache-parallel-runner-01-PLAN.md - Add eval cache keys, deterministic parallel execution, stored-run reuse, and runtime diagnostics.

### Phase 52: Source-Free Candidate Feature Export

**Goal**: Maintainers can export privacy-safe candidate feature rows for learning, diagnostics, and paired analysis.

**Depends on**: Phase 51

**Requirements**: FEATURE-01, FEATURE-02, FEATURE-03, FEATURE-04

**Success Criteria** (what must be TRUE):
1. Eval output can emit candidate feature rows for files, symbols, tests, docs, commits, memory cards, feedback, and graph candidates.
2. Feature rows include signal scores, ranks, role/type metadata, graph distance, history/test/memory/feedback metadata, and source-free labels.
3. Feature exports reject source snippets, prompt text, issue descriptions, terminal logs, stack traces, and secret-bearing values.
4. CLI/storage commands can list, inspect, compare, and delete feature exports by run and privacy status.

**Plans**: 1 plan

Plans:
- [ ] 52-source-free-candidate-feature-export-01-PLAN.md - Create feature export contracts, privacy checks, storage operations, and CLI/report surfaces.

### Phase 53: Paired Baseline & Ablation Analysis

**Goal**: Maintainers can compare ctxpack against lexical, no-context, and signal-ablation baselines with honest verdicts.

**Depends on**: Phase 52

**Requirements**: BASELINE-01, BASELINE-02, BASELINE-03, BASELINE-04

**Success Criteria** (what must be TRUE):
1. Reports compare default ctxpack ranking against lexical, no-context, graph-only, semantic-only, history-only, test-only, memory-only, and feedback-weighted variants on the same corpus.
2. Reports include Recall@K, precision proxy, test recall, token ROI, validation coverage, missed-family taxonomy, signal saturation, runtime, and privacy status.
3. Verdicts classify each comparison as lift, neutral, regression, or insufficient evidence using configured thresholds.
4. Lexical parity and lexical regression are explicitly called out.

**Plans**: 1 plan

Plans:
- [ ] 53-paired-baseline-ablation-analysis-01-PLAN.md - Add paired comparison reports, threshold verdicts, signal saturation diagnostics, and lexical-parity flags.

### Phase 54: Offline Learned Retrieval Policy Experiment

**Goal**: Maintainers can generate, compare, apply, disable, and roll back non-default learned policy profiles.

**Depends on**: Phase 53

**Requirements**: POLICY-01, POLICY-02, POLICY-03, POLICY-04

**Success Criteria** (what must be TRUE):
1. Offline learner proposes retrieval-policy weights from source-free feature exports, historical labels, and feedback/outcome traces.
2. Learned proposals are stored as non-default profiles with training provenance, schema version, metrics, and rollback metadata.
3. Existing policy controls can compare, apply, disable, and roll back learned profiles.
4. Learned profiles cannot become default unless configured baseline thresholds pass.

**Plans**: 1 plan

Plans:
- [ ] 54-offline-learned-retrieval-policy-experiment-01-PLAN.md - Implement source-free policy proposal, profile persistence, comparison, and guarded application.

### Phase 55: Product Proof Gates & v2.3 Release Integration

**Goal**: Maintainers can include bounded v2.3 eval proof in docs and release gates without requiring external repos by default.

**Depends on**: Phases 50-54

**Requirements**: PROOF-01, PROOF-02, PROOF-03, PROOF-04

**Success Criteria** (what must be TRUE):
1. Product proof includes fixed corpus identity, paired baseline verdicts, runtime diagnostics, feature-export privacy, and learned-policy status.
2. Release gate can run a small deterministic v2.3 eval smoke without external repos.
3. RefactoringMiner and multi-repo proof remain optional external gates with reproducible commands and skip reasons.
4. Docs explain that useful context at lexical parity is not the same as world-class lift.

**Plans**: 1 plan

Plans:
- [ ] 55-product-proof-gates-v23-release-integration-01-PLAN.md - Wire v2.3 proof, docs, release-gate smoke, and proof-boundary language.

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| CORPUS-01 | Phase 50 |
| CORPUS-02 | Phase 50 |
| CORPUS-03 | Phase 50 |
| CORPUS-04 | Phase 50 |
| SPEED-01 | Phase 51 |
| SPEED-02 | Phase 51 |
| SPEED-03 | Phase 51 |
| SPEED-04 | Phase 51 |
| FEATURE-01 | Phase 52 |
| FEATURE-02 | Phase 52 |
| FEATURE-03 | Phase 52 |
| FEATURE-04 | Phase 52 |
| BASELINE-01 | Phase 53 |
| BASELINE-02 | Phase 53 |
| BASELINE-03 | Phase 53 |
| BASELINE-04 | Phase 53 |
| POLICY-01 | Phase 54 |
| POLICY-02 | Phase 54 |
| POLICY-03 | Phase 54 |
| POLICY-04 | Phase 54 |
| PROOF-01 | Phase 55 |
| PROOF-02 | Phase 55 |
| PROOF-03 | Phase 55 |
| PROOF-04 | Phase 55 |

**Coverage:** 24/24 v2.3 requirements mapped. No orphaned requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 50 -> 51 -> 52 -> 53 -> 54 -> 55

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 50. Fixed Benchmark Corpus & RefactoringMiner Regression Suite | 1/1 | Complete | 2026-05-19 |
| 51. Historical Eval Cache & Parallel Runner | 1/1 | Complete | 2026-05-19 |
| 52. Source-Free Candidate Feature Export | 0/1 | Not Started | - |
| 53. Paired Baseline & Ablation Analysis | 0/1 | Not Started | - |
| 54. Offline Learned Retrieval Policy Experiment | 0/1 | Not Started | - |
| 55. Product Proof Gates & v2.3 Release Integration | 0/1 | Not Started | - |

---
*Roadmap created: 2026-05-19*
