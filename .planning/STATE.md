---
gsd_state_version: 1.0
milestone: v2.5
milestone_name: Production Retrieval Quality
status: active
last_updated: "2026-05-30T19:40:00Z"
last_activity: 2026-05-30 -- Phase 78 made the broader product-proof gate ceiling-aware and promoted the fixed four-repo proof
progress:
  total_phases: 16
  completed_phases: 16
  total_plans: 7
  completed_plans: 7
  percent: 100
---

# Project State

## Current Position

Phase: 78 - Ceiling-Aware Broader Gate
Plan: 78-ceiling-aware-broader-gate
Status: Complete
Last activity: 2026-05-30 -- Phase 78 accepts safe perfect lexical-ceiling matches in the product-proof gate while keeping ordinary matches and hard runtime failures as blockers. The broader fixed-corpus proof now promotes; RefactoringMiner is a safe ceiling match, while ctxpack, ReAgent, and VeriSchema beat lexical on the non-test context channel.

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

Continue production-readiness work from remaining measured gaps: protected retrieval-target misses in ctxpack/VeriSchema, parser/precision dependency misses, and steady-state latency/quality hardening.

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
- Latest real-client proof: `.planning/e2e/2026-05-30-phase70-real-client-mcp-proof.md`.
- RefactoringMiner still trails lexical on all-file recall because tests are no longer forced into the target-file context budget; this is explicitly recorded in corpus verdict notes.
- Next work should target remaining protected retrieval-target misses in ctxpack/VeriSchema, low-information/multi-area task detection, parser precision, and warm-cache latency characterization.
