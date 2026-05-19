---
gsd_state_version: 1.0
milestone: v2.3
milestone_name: Evaluation Lab & Learned Retrieval Policy
status: in_progress
last_updated: "2026-05-19T00:00:00.000+06:00"
last_activity: 2026-05-19 - Phase 54 completed offline learned retrieval policy experiments
progress:
  total_phases: 6
  completed_phases: 5
  total_plans: 6
  completed_plans: 5
  percent: 83
---

# Project State

## Current Position

Phase: 55
Plan: -
Status: Phase 54 complete; Phase 55 next
Last activity: 2026-05-19 - Phase 54 completed offline learned retrieval policy experiments

## Project Reference

See: `.planning/PROJECT.md`

**Core value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

**Current focus:** v2.3 will make ctxpack's retrieval-quality claims repeatable across fixed corpora, large histories, policy variants, and source-free learned retrieval experiments.

## Active Milestone

v2.3 Evaluation Lab & Learned Retrieval Policy is active.

## Next Step

Plan Phase 55: Product Proof Gates & v2.3 Release Integration.

## Operator Next Steps

- Run `$gsd-plan-phase 55` to create the executable plan for product proof gates and v2.3 release integration.
- Keep RefactoringMiner as the first locked large-history regression target, but do not claim broad product lift from that repo alone.
- Preserve local-first, read-only, source-free learning constraints in every eval artifact.
