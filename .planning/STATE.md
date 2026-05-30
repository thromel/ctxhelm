---
gsd_state_version: 1.0
milestone: v2.5
milestone_name: Production Retrieval Quality
status: active
last_updated: "2026-05-30T15:05:00Z"
last_activity: 2026-05-30 -- Phase 71 reduced ctxpack archive-artifact retrieval noise while preserving the promotion gate
progress:
  total_phases: 10
  completed_phases: 10
  total_plans: 7
  completed_plans: 7
  percent: 100
---

# Project State

## Current Position

Phase: 71 - Archive Artifact Dampening
Plan: 71-archive-artifact-dampening
Status: Complete
Last activity: 2026-05-30 -- Phase 71 dampened `.planning/milestones/**` and `.planning/e2e/**/*.json` lexical archive artifacts without excluding them. Current-history proof still promotes and improves ctxpack context Recall@10 from 0.3117 to 0.5195, file Recall@10 from 0.2909 to 0.4986, and protected miss-rate@10 from 0.2500 to 0.1633 while leaving RefactoringMiner unchanged.

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

Continue production-readiness work from remaining measured gaps: parser/precision dependency misses and broader multi-repo repeated-lift validation.

## Operator Next Steps

- Default local retrieval now passes the fixed two-repo product proof under the channel-aware release gate.
- The gate compares non-test context recall against lexical and checks validation-test recall separately.
- Latest local proof: `.ctxpack/e2e/phase71-archive-artifact-dampening-proof.json` with `releaseGate.decision = promote`.
- Latest real-client proof: `.planning/e2e/2026-05-30-phase70-real-client-mcp-proof.md`.
- RefactoringMiner still trails lexical on all-file recall because tests are no longer forced into the target-file context budget; this is explicitly recorded in corpus verdict notes.
- Next work should target parser precision and broader repeated-lift corpora.
