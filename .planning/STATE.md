---
gsd_state_version: 1.0
milestone: v2.3
milestone_name: Evaluation Lab & Learned Retrieval Policy
status: in_progress
last_updated: "2026-05-19T00:00:00.000+06:00"
last_activity: 2026-05-19 - Phase 53 completed paired baseline and ablation analysis
progress:
  total_phases: 6
  completed_phases: 4
  total_plans: 6
  completed_plans: 4
  percent: 67
---

# Project State

## Current Position

Phase: 54
Plan: -
Status: Phase 53 complete; Phase 54 next
Last activity: 2026-05-19 - Phase 53 completed paired baseline and ablation analysis

## Project Reference

See: `.planning/PROJECT.md`

**Core value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

**Current focus:** v2.3 will make ctxpack's retrieval-quality claims repeatable across fixed corpora, large histories, policy variants, and source-free learned retrieval experiments.

## Active Milestone

v2.3 Evaluation Lab & Learned Retrieval Policy is active.

## Next Step

Plan Phase 54: Offline Learned Retrieval Policy Experiment.

## Operator Next Steps

- Run `$gsd-plan-phase 54` to create the executable plan for offline learned retrieval policy experiments.
- Keep RefactoringMiner as the first locked large-history regression target, but do not claim broad product lift from that repo alone.
- Preserve local-first, read-only, source-free learning constraints in every eval artifact.
