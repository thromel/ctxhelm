# Roadmap: Repo Context Packer

## Overview

This roadmap opens v1.2 Retrieval Quality Proof. v1 and v1.1 proved that ctxpack can safely generate task-conditioned plans, packs, cards, traces, MCP resources, and release artifacts. v1.2 must prove why the product matters: agents should get better context, with fewer irrelevant reads and better validation guidance, on real repositories under fixed budgets.

The milestone uses RefactoringMiner as the primary large-history proof target and adds at least one more real repository to avoid overfitting. All benchmark artifacts must remain source-free.

## v1.2 Retrieval Quality Proof

## Phases

**Phase Numbering:**
- Integer phases (9, 10, 11, 12): Planned v1.2 work
- Decimal phases (10.1, 10.2): Urgent insertions if needed

- [ ] **Phase 9: Benchmark Harness & Corpus Contracts** - Maintainers can define reproducible, source-free benchmark suites over real repositories.
- [ ] **Phase 10: Fixed-Budget Retrieval Metrics & Baselines** - Maintainers can compare ctxpack retrieval against lexical and no-context baselines with stable metrics.
- [ ] **Phase 11: Retrieval Gap Taxonomy & Regression Trends** - Maintainers can turn repeated misses into source-free failure families and trend reports.
- [ ] **Phase 12: Product Proof Report & Adoption Gate** - Users can see why ctxpack is useful through reproducible reports, docs, and release-gate proof.

## Phase Details

### Phase 9: Benchmark Harness & Corpus Contracts
**Goal**: Maintainers can define reproducible, source-free benchmark suites over RefactoringMiner and at least one additional real repository.
**Depends on**: v1.1 Packaging & Adoption
**Requirements**: BENCH-01, BENCH-02, BENCH-03, BENCH-04
**Success Criteria** (what must be TRUE):
  1. Maintainer can define named benchmark suites with repo path, revision range, max commits, budgets, and role filters without storing source text.
  2. Maintainer can run a bounded historical eval over RefactoringMiner and another configured real repo using reproducible revision ranges.
  3. Benchmark outputs include source-free task labels, changed-file/test labels, privacy status, skipped-path counts, and reproducibility metadata.
  4. Fixtures and docs explain how to add another local repo benchmark without cloud services or global machine assumptions.
**Plans**: 4 plans
Plans:
- [ ] 09-benchmark-harness-corpus-contracts-01-PLAN.md — Define benchmark suite configuration and source-free corpus contracts.
- [ ] 09-benchmark-harness-corpus-contracts-02-PLAN.md — Implement multi-repo bounded benchmark execution.
- [ ] 09-benchmark-harness-corpus-contracts-03-PLAN.md — Persist reproducibility metadata and privacy diagnostics.
- [ ] 09-benchmark-harness-corpus-contracts-04-PLAN.md — Document RefactoringMiner and second-repo benchmark setup.

### Phase 10: Fixed-Budget Retrieval Metrics & Baselines
**Goal**: Maintainers can measure whether ctxpack improves target-file/test retrieval over lexical and no-context baselines at fixed budgets.
**Depends on**: Phase 9
**Requirements**: METR-01, METR-02, METR-03, METR-04, METR-05, ROI-01, ROI-02
**Success Criteria** (what must be TRUE):
  1. Benchmark reports include file Recall@K, test Recall@K, precision-like useful-target ratios, and missing-label summaries for every suite.
  2. Reports compare ctxpack hybrid ranking against lexical-only and no-context/anchor-only baselines under the same candidate and token budgets.
  3. Signal ablations quantify lift from symbols, dependencies, tests, git history, current diff, and docs/cards without leaking source snippets.
  4. Token ROI estimates show useful targets per 1k estimated tokens and pack budget efficiency for brief, standard, and deep options.
  5. CLI and JSON output stay stable enough to feed portfolio/docs tables and future release gates.
**Plans**: 5 plans
Plans:
- [ ] 10-fixed-budget-retrieval-metrics-baselines-01-PLAN.md — Add baseline runners and stable benchmark metric contracts.
- [ ] 10-fixed-budget-retrieval-metrics-baselines-02-PLAN.md — Add fixed-budget recall, useful-target, and missing-label metrics.
- [ ] 10-fixed-budget-retrieval-metrics-baselines-03-PLAN.md — Add signal ablation and token ROI reporting.
- [ ] 10-fixed-budget-retrieval-metrics-baselines-04-PLAN.md — Expose benchmark reports through CLI JSON and Markdown outputs.
- [ ] 10-fixed-budget-retrieval-metrics-baselines-05-PLAN.md — Validate metrics against RefactoringMiner and the second real repo.

### Phase 11: Retrieval Gap Taxonomy & Regression Trends
**Goal**: Maintainers can understand repeated retrieval misses and track whether changes improve or regress context quality over time.
**Depends on**: Phase 10
**Requirements**: GAP-01, GAP-02, GAP-03, GAP-04, REG-01, REG-02
**Success Criteria** (what must be TRUE):
  1. Repeated missing files/tests are grouped by source-free families such as role, package, path pattern, signal gap, rename/delete status, and generated/sensitive policy.
  2. Reports identify which missing families point to storage, semantic retrieval, parser precision, test mapping, or history-ranking gaps.
  3. Maintainer can compare two benchmark runs and see deltas for recall, token ROI, signal ablations, skipped files, and repeated failure families.
  4. Regression summaries can fail a check when selected metrics drop beyond configured thresholds.
  5. Gap and regression outputs are source-free and suitable for planning v1.3, v1.4, and v1.5 scope.
**Plans**: 4 plans
Plans:
- [ ] 11-retrieval-gap-taxonomy-regression-trends-01-PLAN.md — Normalize source-free gap families across benchmark suites.
- [ ] 11-retrieval-gap-taxonomy-regression-trends-02-PLAN.md — Map gap families to future milestone recommendations.
- [ ] 11-retrieval-gap-taxonomy-regression-trends-03-PLAN.md — Add benchmark run comparison and regression thresholds.
- [ ] 11-retrieval-gap-taxonomy-regression-trends-04-PLAN.md — Validate trend reports on repeated real-repo benchmark runs.

### Phase 12: Product Proof Report & Adoption Gate
**Goal**: Users can see a credible, reproducible answer to "why use ctxpack instead of the agent's native search?"
**Depends on**: Phase 11
**Requirements**: PROOF-01, PROOF-02, PROOF-03, PROOF-04, PROOF-05
**Success Criteria** (what must be TRUE):
  1. README or docs include a concise source-free proof report with benchmark setup, baseline comparison, headline metrics, and limitations.
  2. `ctxpack eval` exposes a maintainer-friendly command path that reproduces the product proof on configured local repos.
  3. Release gate can optionally run a bounded benchmark smoke and fail on report-generation or privacy regressions.
  4. The proof report states when ctxpack helps, when it does not, and what future milestones address remaining gaps.
  5. Planning docs for v1.3-v2.1 are updated from measured gaps instead of speculative feature desire.
**Plans**: 4 plans
Plans:
- [ ] 12-product-proof-report-adoption-gate-01-PLAN.md — Generate a source-free product proof report from benchmark artifacts.
- [ ] 12-product-proof-report-adoption-gate-02-PLAN.md — Document benchmark setup, interpretation, limitations, and "why ctxpack" messaging.
- [ ] 12-product-proof-report-adoption-gate-03-PLAN.md — Add optional benchmark smoke to release/adoption gates.
- [ ] 12-product-proof-report-adoption-gate-04-PLAN.md — Update future milestone requirements from measured gap evidence.

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| BENCH-01 | Phase 9 |
| BENCH-02 | Phase 9 |
| BENCH-03 | Phase 9 |
| BENCH-04 | Phase 9 |
| METR-01 | Phase 10 |
| METR-02 | Phase 10 |
| METR-03 | Phase 10 |
| METR-04 | Phase 10 |
| METR-05 | Phase 10 |
| ROI-01 | Phase 10 |
| ROI-02 | Phase 10 |
| GAP-01 | Phase 11 |
| GAP-02 | Phase 11 |
| GAP-03 | Phase 11 |
| GAP-04 | Phase 11 |
| REG-01 | Phase 11 |
| REG-02 | Phase 11 |
| PROOF-01 | Phase 12 |
| PROOF-02 | Phase 12 |
| PROOF-03 | Phase 12 |
| PROOF-04 | Phase 12 |
| PROOF-05 | Phase 12 |

**Coverage:** 22/22 v1.2 requirements mapped. No orphaned requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 9 -> 10 -> 11 -> 12

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 9. Benchmark Harness & Corpus Contracts | 0/4 | Pending | — |
| 10. Fixed-Budget Retrieval Metrics & Baselines | 0/5 | Pending | — |
| 11. Retrieval Gap Taxonomy & Regression Trends | 0/4 | Pending | — |
| 12. Product Proof Report & Adoption Gate | 0/4 | Pending | — |
