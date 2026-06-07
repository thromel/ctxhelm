# Phase 313 - R&D Completion Gate

## Scope

Phase 312 confirmed that Claude cross-agent replication is still blocked by
client availability, not by a ctxhelm retrieval result. This phase records the
strict gate for calling the current R&D effort complete, so future work has a
clear stop condition.

## Gate

The R&D effort is complete only when all current-head checks below are true:

1. **Release and docs are green.** CI, release docs, and the local release gate
   pass on the current head, and the worktree is clean after push.
2. **Codex outcome proof is preserved.** The latest Codex R&D breadth suite is
   comparable and reports `ctxhelm_improved` with no ctxhelm evidence misses,
   evidence-only targets, under-read targets, forbidden commands, client
   failures, or rate limits.
3. **Memory proof remains guarded.** The memory lane preserves the Phase 253/255
   target-consumption and irrelevant-read evidence, or a newer broader proof
   supersedes it without regression.
4. **Semantic is resolved explicitly.** Either semantic/default promotion has a
   no-regress lift proof on the agreed stable corpus, or semantic is explicitly
   closed as diagnostic-only with the current rejected branches recorded and no
   higher-leverage same-family experiment pending.
5. **Claude cross-agent state is not ambiguous.** Either a fresh Claude paired
   R&D suite is comparable and passes its read-only/source-free boundary, or the
   completion audit explicitly classifies Claude as an external availability
   blocker based on current source-free evidence.
6. **Known client/auth blockers are scoped.** Cursor or other optional-client
   blockers are documented as external auth/availability issues, not hidden
   retrieval-quality failures.
7. **The final audit says complete without caveats.** The R&D completion audit
   must state which gate rows are satisfied, superseded, or externally blocked;
   unresolved retrieval or policy questions cannot be buried in narrative text.

## Current Status

The gate is not fully satisfied today.

- Satisfied: release/docs/CI are green at `3a5f436`; Codex Phase 310/311 closes
  the measured evidence-only consumption gap; memory Phase 253/255 remains the
  active guard; optional client/auth blockers are documented.
- Not satisfied as complete: semantic/default promotion is still diagnostic-only
  with no accepted closeout decision, and Claude has current rate-limit evidence
  rather than a comparable paired suite or a formal external-blocker closeout.

## Decision

Use this gate before marking the active R&D goal complete. Further semantic work
should only proceed if it introduces a materially different source-free feature
family or produces a closeout decision; repeated same-family relaxations remain
rejected by Phase 302/303. Further Claude work should retry the same paired
suite only when the client no longer emits rate-limit events.
