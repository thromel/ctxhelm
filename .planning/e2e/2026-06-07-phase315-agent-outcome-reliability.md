# Phase 315 - Agent Outcome Reliability

## Goal

Make ctxhelm's accepted agent outcome proof more efficient and legible while
preserving release hygiene, read-only boundaries, and source-free reporting.

Phase 315 starts from the Phase 314 closeout state:

- Codex Phase 310/311 is the current comparable outcome proof.
- Memory Phase 253/255 is the active target-consumption and read-efficiency
  guard.
- Semantic/default promotion is closed as diagnostic-only for the previous R&D
  scope.
- Claude is an external availability blocker until source-free paired-suite
  availability clears.

## Non-Goals

- Do not reopen semantic default promotion.
- Do not add autonomous agent behavior.
- Do not add MCP tools unless current tools/resources cannot express the
  behavior.
- Do not weaken read-only or source-free accounting.
- Do not treat rate-limited or client-failed runs as retrieval-quality evidence.

## Work Items

### 1. Release-Smoke Hygiene

Make `scripts/smoke-distribution-metadata.sh` temp-path clean by default so the
release gate does not leave `.ctxhelm/distribution-metadata-smoke.json` dirty
after rebuilding archives.

Acceptance:

- `scripts/release-gate.sh` passes from a clean checkout.
- The worktree remains clean after release gate.
- CI release gate still passes.
- Archive/Homebrew/release-doc proof is not weakened.

### 2. Retry Read-Cost Metrics

Add source-free retry cost accounting to Codex agent-run reports:

- retry-triggered lane count;
- retry-selected lane count;
- read-file count before/after retry;
- irrelevant-read count before/after retry;
- target-read coverage before/after retry;
- evidence-only targets before/after retry.

Acceptance:

- Phase 311-style selected reports still reach target-read coverage `1.0`.
- Evidence-only targets remain zero after selected retry.
- Reports quantify read-cost deltas without raw prompts, transcripts, MCP
  traffic, command output, or source text.

### 3. Codex Retry-Cost Repeat

Repeat the four-task Codex R&D breadth suite with retry-cost accounting and
compare target-read coverage, evidence-only targets, read-file count,
irrelevant-read count, retry trigger/selection rates, forbidden commands,
client failures, and rate limits.

Acceptance:

- Target-read coverage remains at Phase 311 level.
- Evidence-only targets remain zero.
- Retry does not materially regress irrelevant reads.
- Memory Phase 253/255 guard remains intact.

### 4. Claude Availability Split

Separate tiny-prompt availability from paired-suite comparability.

Acceptance:

- Reports distinguish tiny-prompt availability from paired-suite availability.
- Rate-limited runs cannot produce retrieval-quality claims.
- Phase 312 task set remains reusable without changing comparison rules.

### 5. Optional Claude Phase 312 Rerun

Rerun the exact Phase 312 paired suite only if source-free availability no
longer records Claude rate limits/client failures.

Acceptance:

- Baseline plus at least one ctxhelm-assisted lane is comparison eligible.
- Required ctxhelm calls are observed.
- No forbidden shell/edit/write calls in read-only lanes.
- No client failures or rate limits.
- ctxhelm improves target-read coverage or irrelevant-read count without losing
  target coverage.

### 6. Proof Dashboard / Inspector Summary

Expose proof reports in a human-readable source-free summary. A static local
report is enough for this phase.

Minimum sections:

1. outcome claim;
2. comparable task/lane count;
3. target-read coverage;
4. evidence-only targets;
5. irrelevant-read count;
6. retry cost;
7. memory guard status;
8. client failures/rate limits;
9. forbidden boundary events;
10. source-free/privacy status;
11. recommended next action.

Acceptance:

- No source text, raw prompt, raw transcript, or raw MCP traffic.
- Can render Phase 311, Phase 253, Phase 255, and Phase 312 reports.
- Clearly labels degraded/non-comparable reports.

## First PR

Start with release-smoke hygiene because it is low risk, improves maintainer
confidence, and directly addresses a known post-validation dirty-worktree
problem.
