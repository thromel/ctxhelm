---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: Local Semantic Retrieval
status: Roadmap ready
last_updated: "2026-05-15T18:49:10.489Z"
last_activity: 2026-05-16
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Current Position

Phase: 17 — Semantic Provider & Privacy Contracts
Plan: Ready for phase discussion/planning
Status: Roadmap ready
Last activity: 2026-05-16 — Milestone v1.4 initialized with requirements and roadmap

## Project Reference

See: `.planning/PROJECT.md`

**Core value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

**Current focus:** v1.4 Local Semantic Retrieval.

## Active Milestone

v1.4 adds optional local semantic retrieval as a measured signal inside the existing context compiler:

- configure local embedding providers explicitly while keeping semantic retrieval disabled by default;
- store vector metadata locally with privacy labels and no raw source snippets;
- generate vector candidates for conceptual tasks and fuse them with lexical, graph, test, history, and active-context signals;
- prove semantic lift or regression through fixed-budget, source-free benchmark reports;
- keep cloud embeddings and reranking visibly opt-in and disabled by default.

## Next Step

Start Phase 17: Semantic Provider & Privacy Contracts.

## Operator Next Steps

- Run `$gsd-discuss-phase 17` to clarify implementation approach.
- Then run `$gsd-plan-phase 17` to create the first execution plan.
