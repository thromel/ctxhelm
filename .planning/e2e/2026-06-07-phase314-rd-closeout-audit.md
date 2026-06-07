# Phase 314 - R&D Closeout Audit

## Scope

Phase 313 defined the completion gate for the active R&D effort. This phase
applies that gate to current evidence and records final closeout decisions for
the two rows that were still open: semantic/default promotion and Claude
cross-agent replication.

## Gate Status

| Gate row | Status | Evidence |
| --- | --- | --- |
| Release and docs are green | Satisfied pending latest push CI | Local `scripts/check-release-docs.sh` and `git diff --check` pass before commit; current main CI must remain the authoritative post-push check. |
| Codex outcome proof is preserved | Satisfied | `.ctxhelm/e2e/phase311-agent-run-codex-rd-breadth-suite-consumption-retry.json` reports `passed`, `ctxhelm_improved`, 4 comparison-eligible tasks, 16 comparable ctxhelm lanes, target-read delta average `0.2916666666666667`, no evidence misses, no evidence-only targets, no under-read targets, no client failures, and no rate limits. |
| Memory proof remains guarded | Satisfied | `.ctxhelm/e2e/phase253-codex-memory-outcome-suite.json` reports memory target-read improvement in 3/3 pairs; `.ctxhelm/e2e/phase255-codex-memory-efficiency-suite.json` preserves that 3/3 target-read improvement and improves memory irrelevant reads in 3/3 pairs. |
| Semantic is resolved explicitly | Satisfied by diagnostic-only closeout | Phases 252-303 record the rejected semantic/reranker/default-promotion branches. The current same-family surface has no accepted no-regress default-lift proof, and Phase 302/303 show the last retention-separator branch is structurally weak. Semantic remains local, opt-in, and diagnostic-only for this R&D scope. |
| Claude cross-agent state is not ambiguous | Satisfied by external availability closeout | `.ctxhelm/e2e/phase312-agent-client-availability.json` records Claude Code `2.1.163` as `rate_limited`; `.ctxhelm/e2e/phase312-agent-run-claude-rd-breadth-suite-preflight-disabled.json` records `degraded`, `insufficient_comparable_lanes`, zero comparison-eligible tasks, zero comparable ctxhelm lanes, client failures observed, and rate limits observed. |
| Known client/auth blockers are scoped | Satisfied | Cursor remains auth-blocked locally; Claude is rate-limited; both are external client availability/auth states rather than retrieval-quality failures. |
| Final audit states every row | Satisfied by this document and the updated completion audit | This table classifies every row as satisfied, diagnostic-only closeout, or external availability closeout. |

## Semantic Closeout

Semantic/default promotion is closed as diagnostic-only for the active R&D
effort.

The rejected same-family branches are already recorded in the completion audit:

- richer semantic documents and path-derived facets;
- global semantic-corroborated fusion;
- query-family, path-family, support-profile, and supported-shape routing;
- broader local-fastembed caps;
- source-role, candidate-path, sibling-path, and generic concept query hints;
- Jina and MiniLM model-swap promotion;
- semantic next-read promotion;
- learned-policy, cross-repo aggregation, and retention-separator relaxations.

The current evidence does not support runtime/default semantic promotion:
supported-shape recovery is safe but sparse, supported-profile precision is too
low, and strict held-out retention separation has zero eligible train families.
Phase 303 confirms this is not just a threshold artifact: only 8 of 117 train
families have positive margin, and relaxed rows recover targets only by
inserting more non-targets.

Future semantic work should be treated as new R&D only if it introduces a
materially different source-free feature family or a new evaluation corpus. It
is not a pending same-family obligation for this active goal.

## Claude Closeout

Claude cross-agent replication is closed as an external availability blocker for
this active R&D effort.

The current state is not ambiguous: small manual prompts may return, but both
the source-free availability check and the full paired Claude suite record
rate-limit/client-failure evidence. The paired suite is therefore not
comparable and cannot prove or disprove ctxhelm retrieval quality. The correct
future action remains `retry_real_client_when_available`, not another
retrieval change.

## Decision

The active R&D effort is closed under the Phase 313 gate once the latest push CI
is green. The product remains shippable on the current proof base: Codex
outcome, memory, release, and docs evidence are current; semantic is explicitly
diagnostic-only; Claude/Cursor gaps are external availability/auth states.
