# Next Round R&D Brief - Post Phase 314

## Purpose

This document summarizes the product state after the completed R&D effort and
sets up a clean next round. It is intended for a new engineer or agent starting
from current `main` after Phase 314.

The important framing: the last R&D effort is complete under the Phase 313/314
gate, but completion does not mean every idea was promoted. It means the
current product has a strong shippable proof base, the accepted improvements are
guarded, and the rejected or blocked branches are explicitly scoped.

## Current Product State

ctxhelm is a local-first, read-only context compiler for coding agents. It helps
agents decide what to inspect first by ranking source files, tests, graph
neighbors, git-history hints, memory cards, semantic metadata, constraints, and
progressive context packs without logging source text or taking over edits.

Current proof snapshot:

| Area | State | Evidence |
| --- | --- | --- |
| Release and install | Shippable | Public `v1.1.12` archive/Homebrew path is documented; local and CI release gates pass on current head. |
| Product retrieval | Strong | README records four-repo product proof with zero protected target misses and agent-evidence Recall@10 delta `+0.19379663` over lexical. |
| Codex real-agent outcome | Strong and current | `.ctxhelm/e2e/phase311-agent-run-codex-rd-breadth-suite-consumption-retry.json` is `passed`, `ctxhelm_improved`, with 4 comparison-eligible tasks and 16 comparable ctxhelm lanes. |
| Memory | Guarded | Phase 253 proves memory target-read lift in 3/3 pairs; Phase 255 preserves that and improves memory irrelevant reads in 3/3 pairs. |
| Semantic/default promotion | Closed diagnostic-only | Phases 252-303 record repeated negative or too-sparse results; Phase 314 closes semantic as diagnostic-only for the completed R&D scope. |
| Claude cross-agent proof | External availability blocker | Phase 312 records Claude Code `2.1.163` rate limits, zero comparable lanes, and `insufficient_comparable_lanes`. |
| Cursor proof | External auth blocker | Cursor setup/protocol proof exists; real client proof remains local-auth blocked. |
| Privacy/source-free boundary | Preserved | Release gate, docs, smoke scripts, reports, and agent-run artifacts continue to record local-only/source-free status. |

## What Improved

### 1. Real Codex Outcome Proof Became Stronger

Earlier Codex runs showed ctxhelm could surface the right files while the agent
sometimes failed to read every surfaced target. That gap matters because
retrieval evidence is not enough if the agent only discovers a target path but
does not consume the current file.

Phase 310/311 added a bounded target-consumption retry to the Codex harness.
The retry triggers only when a ctxhelm-assisted lane is otherwise eligible and
has source-free `ctxhelmEvidenceOnlyTargetCount > 0`. The selected report keeps
retry metadata without storing raw prompts, transcripts, MCP traffic, command
output, or source text.

Accepted proof:

- Phase 311 breadth suite: 4/4 comparison-eligible tasks.
- 16 comparable ctxhelm lanes.
- Outcome claim: `ctxhelm_improved`.
- Average target-read coverage: baseline `0.7083333333333333`, all ctxhelm
  lanes `1.0`.
- No evidence misses, evidence-only targets, under-read targets, forbidden
  commands, client failures, or rate limits.

### 2. Memory Moved From Useful to Guarded

Memory is no longer just a local feature with plausible utility. It has
source-free Codex outcome evidence and efficiency evidence:

- Phase 253: memory target-read improvement in 3/3 cross-repo pairs.
- Phase 255: target-read improvement remains 3/3 and memory irrelevant-read
  improvement becomes 3/3.

This gives the next round a clear guardrail: memory changes must preserve both
target consumption and read efficiency unless a broader proof supersedes the
existing guard.

### 3. Agent-Run Reports Became More Honest

The R&D line improved the reporting contract, not just retrieval behavior.
Reports now distinguish:

- surfaced target versus natively read target;
- evidence misses versus evidence-only targets;
- target-read coverage versus target discovery;
- required ctxhelm call compliance;
- client failures and rate limits;
- forbidden read-only boundary violations;
- recommended next R&D actions.

This is a product improvement because it prevents false success claims. The
Claude Phase 312 suite is the clearest example: some lanes did work and saw
ctxhelm tool calls, but the suite correctly stays non-comparable because every
lane records rate-limit/client-failure evidence.

### 4. Release and Distribution Proof Stayed Green

The product remains release-gated and shippable:

- local `scripts/release-gate.sh` passes from a clean checkout;
- GitHub CI run `27092725574` passed both workspace validation and release gate
  smoke on `46d82fa`;
- release docs contract passes;
- archive, Homebrew, first-pack, MCP, setup, inspector, semantic, precision,
  governor, workspace, and agent-native fallback smokes are covered.

### 5. Semantic Work Produced Better Diagnostics

Semantic did not earn default promotion, but it produced useful diagnostic
surfaces:

- corrected Jina code-model contract with 768 dimensions and provider-specific
  query/passage text;
- candidate-missed support profiles;
- supported semantic candidate profiles;
- retained/dropped candidate summaries;
- train/test retention separator reports;
- margin/threshold sweeps.

Those diagnostics are useful for future R&D, but they are not current runtime
promotion evidence.

### 6. Product Positioning Became Clearer

The current product story is sharper:

- ctxhelm is not an agent replacement.
- It is a source-free context steering layer.
- Its best current proof is agent-evidence retrieval plus real Codex
  consumption.
- Semantic remains optional/diagnostic.
- External client failures are not hidden as retrieval failures.

## What Regressed Or Did Not Promote

There were no accepted production regressions in the final closeout. The final
validated state is green. The regressions below are R&D findings or tradeoffs
that were rejected, contained, or classified.

### 1. Semantic Default Promotion Regressed In Multiple Branches

Several semantic/reranker experiments caused target churn, lower recall, or
non-target insertion. These were rejected and not promoted:

- rich symbol/dependency semantic documents;
- path-derived Python module/package facets;
- global semantic-corroborated reranking;
- query-family and path-family routing;
- support-profile and supported-shape routing as runtime policy;
- broader local-fastembed caps;
- generic source-role query hints;
- candidate sibling/path-concept query hints;
- MiniLM12 model swaps;
- semantic next-read promotion;
- cross-repo learned-policy aggregation;
- retention-separator relaxations.

Most concrete negative pattern: semantic could sometimes find useful candidates,
but final top-K promotion often inserted docs/planning/scripts/paper non-targets
or displaced source/test targets. The supported-shape predictor was safe but too
sparse: across eight stable slices it produced one lift, zero regressions, and
zero default-only target churn, which is not enough for runtime/default
promotion.

### 2. Semantic Retention Separator Was Structurally Weak

The final semantic separator line did not fail because of a single threshold.
Phase 302 found zero eligible train families across four repos. Phase 303 showed
only 8 of 117 train families had positive margin, and relaxed rows recovered
targets only while inserting more non-targets.

Decision: semantic/default promotion is closed as diagnostic-only for this R&D
scope. A next round should not keep relaxing the same separator unless it brings
a materially new source-free feature family or corpus.

### 3. Codex Retry Improves Consumption, Not Efficiency

The retry layer is accepted because it closes evidence-only target consumption.
It is not an efficiency improvement. Retrying can increase read-file and
irrelevant-read counts in some lanes.

Guardrail: treat Phase 311 as consumption enforcement proof. Do not use it to
claim memory/read efficiency; Phase 255 remains the memory-efficiency guard.

### 4. Claude Cross-Agent Proof Regressed To Availability-Blocked

Older Claude workflow proof exists historically, but current Claude Code
`2.1.163` is rate-limited in source-free availability and in the preflight-
disabled four-task paired suite.

Phase 312 suite state:

- status `degraded`;
- outcome claim `insufficient_comparable_lanes`;
- zero comparison-eligible tasks;
- zero comparable ctxhelm lanes;
- client failures observed;
- rate limits observed;
- memory-lane forbidden `Bash` calls observed.

Decision: this is an external availability/read-only-boundary issue, not
retrieval-quality evidence. Future work should retry the same suite only when
Claude no longer emits rate-limit events.

### 5. Cursor Remains Auth-Blocked

Cursor setup and deterministic protocol proof are present. Real Cursor Agent
tool-call proof remains blocked by local auth state. This is external client
availability, not a ctxhelm retrieval regression.

### 6. Release Gate Smoke Can Update Generated Distribution Metadata

Running the full local release gate can rewrite
`.ctxhelm/distribution-metadata-smoke.json` with a newly built archive hash. The
final proof run passed from a clean checkout, and the generated local mutation
was restored afterward. This is not a product behavior regression, but the next
round may want to make that smoke write to a temp path by default to avoid
post-validation dirty worktrees.

## Evidence Inventory

Use these artifacts as the current authoritative base:

| Artifact | Meaning |
| --- | --- |
| `.planning/e2e/2026-06-07-phase314-rd-closeout-audit.md` | Final closeout of the completed R&D scope. |
| `.planning/e2e/2026-06-07-phase313-rd-completion-gate.md` | Completion gate used to decide whether the scope was done. |
| `.ctxhelm/e2e/phase311-agent-run-codex-rd-breadth-suite-consumption-retry.json` | Current Codex real-agent outcome proof. |
| `.ctxhelm/e2e/phase253-codex-memory-outcome-suite.json` | Memory target-consumption proof. |
| `.ctxhelm/e2e/phase255-codex-memory-efficiency-suite.json` | Memory target-consumption plus read-efficiency proof. |
| `.ctxhelm/e2e/phase312-agent-client-availability.json` | Current Claude/Codex client availability snapshot. |
| `.ctxhelm/e2e/phase312-agent-run-claude-rd-breadth-suite-preflight-disabled.json` | Current Claude paired-suite attempt, degraded by rate limits. |
| `.planning/e2e/2026-06-06-rd-completion-audit.md` | Broad audit of implemented, rejected, and closed R&D branches. |
| `README.md` | Product proof snapshot and public positioning. |

## Recommended Next R&D Round

The next round should not reopen the completed Phase 314 scope by default.
Treat it as a new R&D cycle with explicit hypotheses and promotion gates.

### Theme A - Cross-Agent Outcome Replication

Goal: prove ctxhelm helps more than Codex, without weakening comparability.

Candidate work:

1. Retry the Phase 312 Claude paired suite only after source-free availability
   shows Claude is not rate-limited.
2. Preserve the same four-task R&D suite for comparability.
3. Keep read-only forbidden-tool accounting strict.
4. Add a report field that distinguishes "client can answer tiny prompt" from
   "client can run comparable paired suite."
5. Explore OpenCode real-client outcome proof if it can produce
   machine-checkable read/tool evidence.

Promotion gate:

- baseline plus at least one ctxhelm-assisted lane is comparison eligible;
- required ctxhelm calls are observed;
- no forbidden shell/edit/write calls in read-only lanes;
- no client failures or rate limits;
- ctxhelm improves target-read coverage or irrelevant-read count without losing
  target coverage.

### Theme B - Semantic As A New Feature Family

Goal: avoid repeating the rejected same-family semantic work while still using
the diagnostics learned from Phases 252-303.

Candidate work:

1. Define a new source-free feature family before coding. Examples:
   - structured task-intent classifier trained only on source-free facets;
   - package/API role graph from safe path/symbol metadata;
   - validation-aware semantic slots that cannot displace source/test floors;
   - file-neighborhood summaries based on non-source metadata only.
2. Start with offline historical eval, not runtime/default behavior.
3. Pre-register the no-regress and precision bar before running the proof.
4. Keep Jina as a diagnostic backend only unless runtime ratio and no-regress
   lift improve materially.

Promotion gate:

- stable corpus includes ctxhelm, ReAgent, RefactoringMiner, and VeriSchema;
- no default-only target churn;
- no source/test protected target regression;
- repeated lift across more than a singleton thin cell;
- runtime ratio acceptable under product-proof thresholds;
- privacy status remains local-only/source-free.

Anti-goals:

- do not relax the Phase 302/303 retention separator merely to force
  applications;
- do not promote supported-shape from the singleton VeriSchema lift;
- do not add broader semantic document text or source-derived facets.

### Theme C - Agent Consumption Efficiency

Goal: keep the Phase 311 consumption win while reducing retry/read overhead.

Candidate work:

1. Measure retry-trigger rate, retry-selected rate, read-file count, and
   irrelevant-read count across repeated Codex runs.
2. Add retry precision metrics to the agent-run report.
3. Test whether retry prompts can be narrower without losing target reads.
4. Compare retry behavior against memory lane and standard lane separately.

Promotion gate:

- target-read coverage remains at Phase 311 level;
- evidence-only targets remain zero in selected reports;
- irrelevant reads improve or do not regress materially;
- no forbidden commands/client failures/rate limits.

### Theme D - Release/Smoke Hygiene

Goal: reduce maintenance friction around validation without changing product
behavior.

Candidate work:

1. Make `scripts/smoke-distribution-metadata.sh` write generated metadata to a
   temp path by default during release-gate runs, or make the expected output
   deterministic across local archive rebuilds.
2. Add an explicit release-gate assertion that the worktree is clean after
   validation, except for documented opt-in generated artifacts.
3. Improve docs around optional real-client skips versus required real-client
   proof.

Promotion gate:

- local `scripts/release-gate.sh` passes from a clean checkout;
- the worktree remains clean after the gate;
- GitHub release gate smoke still passes;
- no release artifact audit weakening.

### Theme E - Product UX And Adoption

Goal: convert the strong proof base into easier adoption.

Candidate work:

1. Improve first-pack onboarding messages.
2. Add a concise proof dashboard or inspector page summarizing why files were
   selected and what the agent should read next.
3. Add examples for common agent workflows: bug fix, release proof, test
   failure, broad architecture review, and public commit reproduction.
4. Improve setup diagnostics for Claude/Cursor/OpenCode when real-client proof
   is unavailable.

Promotion gate:

- no source text leakage;
- generated guidance stays thin and dynamic;
- setup-check remains deterministic;
- first-pack smoke and inspector smoke pass.

## Suggested First Three Tasks

1. **Release-smoke hygiene:** make the distribution metadata smoke avoid dirtying
   the worktree during release-gate validation.
2. **Claude availability retry:** rerun source-free availability; if Claude is
   available, rerun the exact Phase 312 paired suite.
3. **Retry efficiency report:** add aggregate retry read-cost metrics to Codex
   agent-run reports and compare Phase 311 behavior against a fresh repeat.

This order starts with low-risk maintenance, then checks whether the external
Claude blocker has cleared, then improves the most important accepted R&D
feature without reopening the rejected semantic surface.

## Non-Negotiable Constraints For The Next Round

- Keep ctxhelm read-only.
- Keep reports source-free and local-only unless an explicit provider policy
  says otherwise.
- Do not store raw prompts, raw transcripts, raw MCP traffic, command output, or
  source text in proof artifacts.
- Do not claim retrieval quality from rate-limited or client-failed runs.
- Do not call semantic default-promoted without a no-regress proof on the stable
  corpus.
- Do not weaken release gate, product proof, or source-sentinel checks to make a
  new branch pass.

## Summary

After R&D, ctxhelm is a shippable product with a strong local-first proof base.
The accepted improvements are Codex target-consumption enforcement, memory
guarding, stronger report honesty, release-gate durability, and clearer product
positioning. The main regressions were contained R&D negatives: semantic
promotion repeatedly produced churn or sparse lift, Claude is currently
availability-blocked, Cursor remains auth-blocked, and retry improves
consumption rather than efficiency.

The next round should focus on cross-agent replication when clients are
available, a materially new semantic feature family if semantic is reopened,
retry/read-cost efficiency, release-smoke hygiene, and adoption UX.
