---
gsd_state_version: 1.0
milestone: v2.3
milestone_name: Evaluation Lab & Learned Retrieval Policy
status: in_progress
last_updated: "2026-05-19T00:00:00.000+06:00"
last_activity: 2026-05-19 - Phase 51 completed historical eval cache reuse, parallel runner, and runtime diagnostics
progress:
  total_phases: 6
  completed_phases: 2
  total_plans: 6
  completed_plans: 2
  percent: 33
---

# Project State

## Current Position

Phase: 52
Plan: -
Status: Phase 51 complete; Phase 52 next
Last activity: 2026-05-19 - Phase 51 completed historical eval cache reuse, parallel runner, and runtime diagnostics

## Project Reference

See: `.planning/PROJECT.md`

**Core value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

**Current focus:** v2.3 will make ctxpack's retrieval-quality claims repeatable across fixed corpora, large histories, policy variants, and source-free learned retrieval experiments.

## Active Milestone

v2.3 Evaluation Lab & Learned Retrieval Policy is active.

## Next Step

Plan Phase 52: Source-Free Candidate Feature Export.

## Operator Next Steps

- Run `$gsd-plan-phase 52` to create the executable plan for source-free candidate feature export.
- Keep RefactoringMiner as the first locked large-history regression target, but do not claim broad product lift from that repo alone.
- Preserve local-first, read-only, source-free learning constraints in every eval artifact.
