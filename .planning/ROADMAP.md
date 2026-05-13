# Roadmap: Repo Context Packer

## Overview

This roadmap hardens ctxpack from a useful post-MVP context broker into a trustworthy, measurable agent-native product. The sequence protects public contracts first, makes every source-bearing read fresh and safe, turns retrieval into an evidence-weighted system that can be evaluated against lexical baselines, and then proves the operational path through real coding-agent clients.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Compatibility Guardrails & Module Boundaries** - Maintainers can protect current CLI, MCP, and JSON behavior while splitting large modules.
- [ ] **Phase 2: Trust Layer & Operational Diagnostics** - Users receive fresh, safe, explainable results instead of stale cache, unsafe snippets, or silent partial failures.
- [ ] **Phase 3: Measured Retrieval Lift & Eval Gates** - ctxpack ranks typed evidence from graph, test, history, symbol, and diff signals and proves lift with source-free evals.
- [ ] **Phase 4: Agent-Native Client Durability** - Real clients use ctxpack reliably through explicit repo arguments, durable pack semantics, and thin dynamic adapter guidance.

## Phase Details

### Phase 1: Compatibility Guardrails & Module Boundaries
**Goal**: Maintainers can evolve ctxpack internals without breaking current CLI, MCP, and public JSON contracts.
**Depends on**: Nothing (first phase)
**Requirements**: CONT-01, CONT-02, CONT-03, CONT-04
**Success Criteria** (what must be TRUE):
  1. Maintainer can run binary-level CLI tests that exercise core commands and verify output shape, repo path handling, and write side effects.
  2. Maintainer can run MCP handler, resource, and prompt tests that verify tool names, resource URI shapes, session behavior, structured content, and error responses.
  3. Maintainer can compare stable JSON shapes for context plans, packs, eval traces, MCP structured content, and CLI outputs before changing internals.
  4. Maintainer can split large modules into focused submodules while existing CLI, MCP, and library behavior remains unchanged.
**Plans**: TBD

### Phase 2: Trust Layer & Operational Diagnostics
**Goal**: Users can trust that ctxpack read paths are fresh, privacy-gated, and explicit about partial or degraded results.
**Depends on**: Phase 1
**Requirements**: SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, DIAG-01, DIAG-02, DIAG-04
**Success Criteria** (what must be TRUE):
  1. User-facing search, planning, symbols, tests, dependency, pack, card, and MCP read paths detect stale inventory before returning results.
  2. User-facing read paths either rebuild stale inventory or return structured stale-cache diagnostics that explain what changed.
  3. Pack, file-resource, and card generation exclude credential, auth, generated, vendored, binary, oversized, unreadable, and non-UTF-8 inputs through a centralized tested policy.
  4. CLI and MCP outputs expose stable diagnostics for weak plans, stale cache, missing or timed-out git, skipped files, parse gaps, and partial graph/test/history coverage.
  5. Context retrieval remains usable in read-only or constrained home-directory environments, with trace/cache writes visible, controllable, and non-fatal when possible.
**Plans**: TBD

### Phase 3: Measured Retrieval Lift & Eval Gates
**Goal**: Users and maintainers can see ranked, attributed evidence that improves over lexical retrieval at fixed budgets and remains source-free in reports.
**Depends on**: Phase 2
**Requirements**: DIAG-03, RETR-01, RETR-02, RETR-03, RETR-04, RETR-05, EVAL-01, EVAL-02, EVAL-03, EVAL-04, EVAL-05, PARS-01, PARS-02, PARS-03
**Success Criteria** (what must be TRUE):
  1. Context planning ranks typed candidates for files, tests, symbols, docs, commits, and config using evidence and per-signal scores before projecting to `ContextPlan`.
  2. Dependency edges, related tests, co-change hints, current-diff anchors, and symbol matches affect ranked targets with source-free attribution for every recommended file and test.
  3. Graph expansion stays budgeted and non-recursive by default, so retrieval lift is measured at fixed context budgets rather than through context bloat.
  4. Maintainer can run frozen historical eval ranges, including large-repo smokes, with reproducible refs, role-aware labels, rename/delete handling, lexical baselines, ranking metrics, and signal ablations.
  5. Eval and checklist reports remain source-free and prompt-free while summarizing retrieval failures by path role, signal gap, and repeated missing-file family.
**Plans**: TBD

### Phase 4: Agent-Native Client Durability
**Goal**: Users can rely on ctxpack from real coding-agent clients without session surprises, wrong-repo behavior, or static context dumps.
**Depends on**: Phase 3
**Requirements**: AGNT-01, AGNT-02, AGNT-03, AGNT-04
**Success Criteria** (what must be TRUE):
  1. Codex CLI and Claude Code smoke scripts verify real MCP `prepare_task` and `get_pack` client paths with explicit `repo` arguments.
  2. User can tell whether MCP pack resources are session-scoped or reconstructed from persisted source-free metadata after server restarts.
  3. Maintainer can test MCP cache growth, reconnect behavior, and wrong-working-directory behavior without relying on manual client inspection.
  4. Generated adapter guidance stays thin and directs agents to dynamic ctxpack calls instead of injecting large static repository context.
**Plans**: TBD

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| CONT-01 | Phase 1 |
| CONT-02 | Phase 1 |
| CONT-03 | Phase 1 |
| CONT-04 | Phase 1 |
| SAFE-01 | Phase 2 |
| SAFE-02 | Phase 2 |
| SAFE-03 | Phase 2 |
| SAFE-04 | Phase 2 |
| SAFE-05 | Phase 2 |
| SAFE-06 | Phase 2 |
| DIAG-01 | Phase 2 |
| DIAG-02 | Phase 2 |
| DIAG-03 | Phase 3 |
| DIAG-04 | Phase 2 |
| RETR-01 | Phase 3 |
| RETR-02 | Phase 3 |
| RETR-03 | Phase 3 |
| RETR-04 | Phase 3 |
| RETR-05 | Phase 3 |
| EVAL-01 | Phase 3 |
| EVAL-02 | Phase 3 |
| EVAL-03 | Phase 3 |
| EVAL-04 | Phase 3 |
| EVAL-05 | Phase 3 |
| AGNT-01 | Phase 4 |
| AGNT-02 | Phase 4 |
| AGNT-03 | Phase 4 |
| AGNT-04 | Phase 4 |
| PARS-01 | Phase 3 |
| PARS-02 | Phase 3 |
| PARS-03 | Phase 3 |

**Coverage:** 31/31 v1 requirements mapped. No orphaned requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Compatibility Guardrails & Module Boundaries | 0/TBD | Not started | - |
| 2. Trust Layer & Operational Diagnostics | 0/TBD | Not started | - |
| 3. Measured Retrieval Lift & Eval Gates | 0/TBD | Not started | - |
| 4. Agent-Native Client Durability | 0/TBD | Not started | - |
