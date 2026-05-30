---
gsd_state_version: 1.0
milestone: v2.5
milestone_name: Production Retrieval Quality
status: active
last_updated: "2026-05-30T15:16:39Z"
last_activity: 2026-05-30 -- Phase 72 broadened repeated-lift validation and improved validation-test recall seeding while preserving the required two-repo promotion gate
progress:
  total_phases: 10
  completed_phases: 10
  total_plans: 7
  completed_plans: 7
  percent: 100
---

# Project State

## Current Position

Phase: 72 - Broader Repeated-Lift Validation
Plan: 72-broader-repeated-lift-validation
Status: Complete
Last activity: 2026-05-30 -- Phase 72 increased the related-test selection budget to 10 and seeded related-test discovery from co-changed/dependency-neighbor source files. The required fixed two-repo proof still promotes, with Test Recall@10 at 1.0 on both required corpora. A broader four-repo probe improved VeriSchema Test Recall@10 from 0.2434 to 0.6614 but still blocks broader promotion because VeriSchema remains below the 0.80 validation-test floor and RefactoringMiner matches rather than beats on the newest-5-commit probe.

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

Continue production-readiness work from remaining measured gaps: large multi-area validation-test recall, parser/precision dependency misses, protected evidence miss-rate, and fixed-corpus broad benchmark selection.

## Operator Next Steps

- Default local retrieval now passes the fixed two-repo product proof under the channel-aware release gate.
- The gate compares non-test context recall against lexical and checks validation-test recall separately.
- Latest required local proof: `.ctxpack/e2e/phase72-two-repo-post-test-seed-proof.json` with `releaseGate.decision = promote`.
- Latest broader probe: `.ctxpack/e2e/phase72-broader-repeated-lift-proof.json` with `releaseGate.decision = block`; VeriSchema Test Recall@10 improved from `0.2434` to `0.6614` but remains below the broader 0.80 floor.
- Latest real-client proof: `.planning/e2e/2026-05-30-phase70-real-client-mcp-proof.md`.
- RefactoringMiner still trails lexical on all-file recall because tests are no longer forced into the target-file context budget; this is explicitly recorded in corpus verdict notes.
- Next work should target large multi-area validation-test ranking, parser precision, protected evidence misses, and converting broader corpora into stable committed fixtures.
