---
gsd_state_version: 1.0
milestone: v2.5
milestone_name: Production Retrieval Quality
status: active
last_updated: "2026-05-31T04:45:00Z"
last_activity: 2026-05-31 -- Phase 89 added fast inventory freshness
progress:
  total_phases: 27
  completed_phases: 27
  total_plans: 7
  completed_plans: 7
  percent: 100
---

# Project State

## Current Position

Phase: 89 - Fast Inventory Freshness
Plan: 89-fast-inventory-freshness
Status: Complete
Last activity: 2026-05-31 -- Phase 89 makes inventory cache-hit freshness metadata-only instead of full source re-hashing. The pinned broader release proof now promotes: RefactoringMiner `8279ms`, ctxpack `8317ms`, ReAgent `4264ms`, and VeriSchema `6590ms`, with Phase 88 quality metrics preserved.

## Project Reference

See: `.planning/PROJECT.md`

**Core value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

**Current focus:** v2.5 Production Retrieval Quality.

## Active Milestone

v2.5 Production Retrieval Quality

Goal: Prove and improve retrieval quality across real repositories so production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline while staying local-first and source-safe.

Planned phases:

- Phase 61: Multi-Repo Quality Baselines (complete)
- Phase 62: Production Local Embedding Quality (complete)
- Phase 63: Reranker And Fusion Promotion (complete)
- Phase 64: Gap-Family Retrieval Improvements (complete)
- Phase 65: v2.5 Product Proof And Release Gate (complete)
- Phase 66: Test Recall Evaluation Channel (complete follow-up)
- Phase 67: Retrievable Target Eval Denominator (complete follow-up)
- Phase 69: Channel-Aware Product Proof Gate (complete follow-up)
- Phase 70: Real-Client MCP Proof Refresh (complete follow-up)
- Phase 71: Archive Artifact Dampening (complete follow-up)
- Phase 72: Broader Repeated-Lift Validation (complete follow-up)
- Phase 73: Broader Fixed-Corpus Fixture (complete follow-up)
- Phase 74: Protected Evidence Diagnostics (complete follow-up)
- Phase 75: Parent-Bounded History And Test Reserve (complete follow-up)
- Phase 76: Parent-Bounded Validation History (complete follow-up)
- Phase 77: Validation Command Coverage (complete follow-up)
- Phase 78: Ceiling-Aware Broader Gate (complete follow-up)
- Phase 79: Protected Target Floors (complete follow-up)
- Phase 80: Unique Symbol Floor Accounting (complete follow-up)
- Phase 81: Warm Cache Latency Proof (complete follow-up)
- Phase 82: Warm Cache Release Gate (complete follow-up)
- Phase 83: Context Divergence Accounting (complete follow-up)
- Phase 84: Broad Scope Dependency Floors (complete follow-up)
- Phase 85: Broad Context Areas (complete follow-up)
- Phase 86: Python Package Re-Export Graph Coverage (complete follow-up)
- Phase 87: Validation Gap Accounting (complete follow-up)
- Phase 88: Broad Source-Area Candidates (complete follow-up)
- Phase 89: Fast Inventory Freshness (complete follow-up)

## Last Completed Milestone

v2.4 Production Semantic & Precision Backends

Goal: Convert semantic and precision retrieval from local scaffolding into measured, policy-gated retrieval-quality improvements without breaking ctxpack's local-first and source-safe contract.

Planned phases:

- Phase 56: Production Local Semantic Backend (complete)
- Phase 57: Precision-Enriched Semantic Documents (complete)
- Phase 58: Query Construction And Hybrid Fusion Controls (complete)
- Phase 59: Provider And Reranker Policy Gates (complete)
- Phase 60: Semantic/Precision Evaluation Gates And Release Proof (complete)

## Next Step

Continue production-readiness work from remaining measured gaps: source candidate generation for parser/precision `no_candidate_signal` families that still lack candidates, plus release-mode proof hardening as new runtime regressions appear.

## Operator Next Steps

- Default local retrieval now passes the fixed two-repo product proof under the channel-aware release gate.
- The gate compares non-test context recall against lexical and checks validation-test recall separately.
- Latest required local proof: `.ctxpack/e2e/phase77-validation-command-coverage-proof.json` with `releaseGate.decision = promote`.
- Latest broader probe: `.ctxpack/e2e/phase78-ceiling-aware-broader-proof.json` with `releaseGate.decision = promote`; RefactoringMiner is accepted as a safe lexical-ceiling `match`, ctxpack/ReAgent/VeriSchema are `beat`, and VeriSchema keeps Effective Validation Recall@10 `1.0`.
- Reproducible broader fixture: `.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json`; latest Phase 78 run reports `releaseGate.decision = promote`.
- Latest protected-evidence diagnostic proof: `.ctxpack/e2e/phase74-protected-evidence-diagnostics-proof.json` with target miss-rate separated from overall protected pressure.
- Latest broader fixed-corpus diagnostic proof: `.ctxpack/e2e/phase74-broader-protected-evidence-diagnostics-proof.json`; RefactoringMiner and ReAgent have zero protected retrieval-target misses on the pinned probe, while ctxpack and VeriSchema still have target misses.
- Latest parent-bounded validation-history proof: `.ctxpack/e2e/phase76-parent-bounded-validation-history-proof.json`; required proof promotes with RefactoringMiner context Recall@10 `0.778` vs lexical `0.741`, ctxpack context Recall@10 `0.444` vs lexical `0.361`, and Test Recall@10 `1.0` on both corpora.
- Latest broader parent-bounded validation-history proof: `.ctxpack/e2e/phase76-broader-parent-bounded-validation-history-proof.json`; broader promotion still blocks, but VeriSchema Test Recall@10 improved from `0.661` to `0.709`.
- Latest validation-command coverage proof: `.ctxpack/e2e/phase77-validation-command-coverage-proof.json`; required proof promotes with RefactoringMiner effective validation recall `1.0`.
- Latest broader validation-command coverage proof: `.ctxpack/e2e/phase77-broader-validation-command-coverage-proof.json`; VeriSchema effective validation recall is `1.0`, resolving the previous validation-test floor failure through broad command coverage.
- Latest ceiling-aware broader proof: `.ctxpack/e2e/phase78-ceiling-aware-broader-proof.json`; broader fixed-corpus promotion passes, but ctxpack and VeriSchema still report non-zero protected retrieval-target misses.
- Latest protected-target floor proof: `.ctxpack/e2e/phase79-protected-target-floors-proof.json`; required proof promotes, RefactoringMiner protected target miss-rate is `0.0`, and ctxpack required protected target miss-rate is `0.0417`.
- Latest broader protected-target floor proof: `.ctxpack/e2e/phase79-broader-protected-target-floors-proof.json`; broader proof promotes, VeriSchema protected target miss-rate is `0.0`, and ctxpack broader protected target miss-rate remains `0.100`.
- Latest unique-symbol floor proof: `.ctxpack/e2e/phase80-unique-symbol-floor-proof.json`; required proof promotes and required protected target miss-rates are `0.0` across measured corpora.
- Latest broader unique-symbol floor proof: `.ctxpack/e2e/phase80-broader-unique-symbol-floor-proof.json`; broader proof promotes and protected target miss-rates are `0.0` across measured corpora.
- Latest warm-cache latency proof config: `.planning/e2e/2026-05-30-phase81-warm-cache-proof-config.json`.
- Latest warm-cache cold proof: `.ctxpack/e2e/phase81-warm-cache-cold-proof.json`; cold proof promotes and populates source-free eval caches.
- Latest warm-cache warm proof: `.ctxpack/e2e/phase81-warm-cache-warm-proof.json`; warm proof promotes with cache hits on all four corpora and `1ms` reported runtime for each cached repo on this run.
- Latest warm-cache release-gate proof: `.planning/e2e/2026-05-30-phase82-warm-cache-gate.md`, `.ctxpack/e2e/phase82-warm-cache-gate-cold-proof.json`, and `.ctxpack/e2e/phase82-warm-cache-gate-warm-proof.json`; warm-cache regressions now block product-proof promotion.
- Latest context-divergence accounting proof: `.planning/e2e/2026-05-30-phase83-context-divergence-accounting.md` and `.ctxpack/e2e/phase83-context-divergence-proof.json`; product-proof verdicts now expose context-vs-all-file deltas and require all-file lexical deficits to be explained by non-regressed context plus validation channels.
- Latest broad-scope dependency proof: `.planning/e2e/2026-05-31-phase84-broad-scope-dependency-floors.md` and `.ctxpack/e2e/phase84-broad-scope-dependency-proof.json`; broad multi-area tasks are now counted separately and get a bounded dependency source floor.
- Latest broad context-area proof: `.planning/e2e/2026-05-31-phase85-broad-context-areas.md` and `.ctxpack/e2e/phase85-context-areas-warm-proof.json`; broad multi-area prepare-task plans and packs now include source-free `contextAreas` that do not perturb target-file recall.
- Latest Python package re-export graph proof: `.planning/e2e/2026-05-31-phase86-python-package-reexports.md`; focused dependency tests pass and broad proof metrics stay flat/non-regressing, proving the remaining VeriSchema gap is selection/budget pressure rather than only missing Python package edges.
- Latest validation gap accounting proof: `.planning/e2e/2026-05-31-phase87-validation-gap-accounting.md` and `.ctxpack/e2e/phase87-validation-gap-accounting-proof.json`; RefactoringMiner validation-command recall improves from `0.0` to `1.0`, validation-covered tests no longer appear as unresolved test-mapping gap summaries, and the rejected broad source-area diversity experiment is documented.
- Latest broad source-area candidate proof: `.planning/e2e/2026-05-31-phase88-broad-source-area-candidates.md` and `.ctxpack/e2e/phase88-broad-source-area-candidates-proof.json`; VeriSchema File Recall@10 improves from `0.17936651` to `0.18449473` and Source Recall@10 improves from `0.30409357` to `0.31067252` without validation or protected-target regression.
- Latest fast inventory freshness proof: `.planning/e2e/2026-05-31-phase89-fast-inventory-freshness.md`, `.ctxpack/e2e/phase89-fast-inventory-freshness-proof.json`, and `.ctxpack/e2e/phase89-fast-inventory-freshness-release-proof.json`; release-mode broader proof promotes while preserving Phase 88 quality metrics.
- Latest real-client proof: `.planning/e2e/2026-05-30-phase70-real-client-mcp-proof.md`.
- RefactoringMiner and ReAgent still trail lexical on all-file recall in the broader proof, but those deficits are now machine-checkable as explained by the context/validation split instead of only prose notes.
- Phase 84 improves VeriSchema Source Recall@10 from `0.249` to `0.304` on the broader fixed corpus while keeping RefactoringMiner, ctxpack, and ReAgent stable.
- Phase 85 intentionally keeps broad fixed-corpus quality metrics flat while improving agent guidance through non-displacing context-area hints; cold proofs block on the existing RefactoringMiner runtime threshold, while the warm-cache proof promotes.
- Phase 86 adds Python package re-export graph candidates, but VeriSchema Recall@10 remains flat because those candidates do not enter the constrained top-10.
- Phase 87 fixes validation accounting rather than target ranking: Java class-selector commands now count as validation coverage, and validation-covered tests no longer inflate test-mapping gap summaries.
- Phase 88 improves broad VeriSchema source/file recall by adding bounded source-area candidates after graph/test seed selection. An earlier variant was rejected because it perturbed raw test recall.
- Phase 89 reduces repeated inventory freshness overhead. Debug broad proof now blocks only on the RefactoringMiner single-commit cold-start diagnostic, while release-mode broad proof promotes and passes `scripts/check-product-proof.py`.
- Next work should target remaining source candidate gaps and keep release-mode proof as the production runtime authority.
