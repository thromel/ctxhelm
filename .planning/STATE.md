---
gsd_state_version: 1.0
milestone: v2.3
milestone_name: Evaluation Lab & Learned Retrieval Policy
status: planned
last_updated: "2026-05-19T00:00:00.000+06:00"
last_activity: 2026-05-19 - v2.3 milestone planned from refreshed external research and prior ctxpack evidence
progress:
  total_phases: 6
  completed_phases: 0
  total_plans: 6
  completed_plans: 0
  percent: 0
---

# Project State

## Current Position

Phase: Not Started
Plan: -
Status: v2.3 Evaluation Lab & Learned Retrieval Policy planned
Last activity: 2026-05-19 - v2.3 milestone planned from refreshed external research and prior ctxpack evidence

## Project Reference

See: `.planning/PROJECT.md`

**Core value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

**Current focus:** v2.3 will make ctxpack's retrieval-quality claims repeatable across fixed corpora, large histories, policy variants, and source-free learned retrieval experiments.

## Active Milestone

v2.3 Evaluation Lab & Learned Retrieval Policy is active.

## Next Step

Plan Phase 50: Fixed Benchmark Corpus & RefactoringMiner Regression Suite.

## Operator Next Steps

- Run `$gsd-plan-phase 50` to create the executable plan for fixed benchmark corpora.
- Keep RefactoringMiner as the first locked large-history regression target, but do not claim broad product lift from that repo alone.
- Preserve local-first, read-only, source-free learning constraints in every eval artifact.
