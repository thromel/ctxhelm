# Roadmap: Repo Context Packer

## Overview

This roadmap hardens ctxpack from a useful post-MVP context broker into a trustworthy, measurable agent-native product. The sequence protects public contracts first, makes every source-bearing read fresh and safe, turns retrieval into an evidence-weighted system that can be evaluated against lexical baselines, and then proves the operational path through real coding-agent clients.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Compatibility Guardrails & Module Boundaries** - Maintainers can protect current CLI, MCP, and JSON behavior while splitting large modules.
- [x] **Phase 2: Trust Layer & Operational Diagnostics** - Users receive fresh, safe, explainable results instead of stale cache, unsafe snippets, or silent partial failures.
- [x] **Phase 3: Measured Retrieval Lift & Eval Gates** - ctxpack ranks typed evidence from graph, test, history, symbol, and diff signals and proves lift with source-free evals.
- [x] **Phase 4: Agent-Native Client Durability** - Real clients use ctxpack reliably through explicit repo arguments, durable pack semantics, and thin dynamic adapter guidance.

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
**Plans**: 4 plans
Plans:
- [x] 01-compatibility-guardrails-module-boundaries-01-PLAN.md — Add binary-level CLI compatibility guardrails.
- [x] 01-compatibility-guardrails-module-boundaries-02-PLAN.md — Lock public JSON and MCP protocol compatibility surfaces.
- [x] 01-compatibility-guardrails-module-boundaries-03-PLAN.md — Split ctxpack-index behind stable crate-root exports.
- [x] 01-compatibility-guardrails-module-boundaries-04-PLAN.md — Split ctxpack-compiler and ctxpack-mcp behind stable facades.

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
**Plans**: 5 plans
Plans:
- [x] 02-trust-layer-operational-diagnostics-01-PLAN.md — Define diagnostics contracts and central source-read policy.
- [x] 02-trust-layer-operational-diagnostics-02-PLAN.md — Add trusted inventory freshness and cache-write diagnostics.
- [x] 02-trust-layer-operational-diagnostics-03-PLAN.md — Wire fresh inventory and source-read diagnostics through index read paths.
- [x] 02-trust-layer-operational-diagnostics-04-PLAN.md — Add compiler plan diagnostics and pack/card source revalidation.
- [x] 02-trust-layer-operational-diagnostics-05-PLAN.md — Expose diagnostics and constrained write behavior through CLI and MCP.

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
**Plans**: 5 plans
Plans:
- [x] 03-measured-retrieval-lift-eval-gates-01-PLAN.md — Define additive typed candidate and attribution contracts.
- [x] 03-measured-retrieval-lift-eval-gates-02-PLAN.md — Rank and project fixed-budget attributed retrieval candidates.
- [x] 03-measured-retrieval-lift-eval-gates-03-PLAN.md — Add frozen historical eval ranges and role/status labels.
- [x] 03-measured-retrieval-lift-eval-gates-04-PLAN.md — Add ranking metrics, ablations, and source-free gap reports.
- [x] 03-measured-retrieval-lift-eval-gates-05-PLAN.md — Validate CLI/MCP compatibility and bounded historical-eval smoke.

### Phase 4: Agent-Native Client Durability
**Goal**: Users can rely on ctxpack from real coding-agent clients without session surprises, wrong-repo behavior, or static context dumps.
**Depends on**: Phase 3
**Requirements**: AGNT-01, AGNT-02, AGNT-03, AGNT-04
**Success Criteria** (what must be TRUE):
  1. Codex CLI and Claude Code smoke scripts verify real MCP `prepare_task` and `get_pack` client paths with explicit `repo` arguments.
  2. User can tell whether MCP pack resources are session-scoped or reconstructed from persisted source-free metadata after server restarts.
  3. Maintainer can test MCP cache growth, reconnect behavior, and wrong-working-directory behavior without relying on manual client inspection.
  4. Generated adapter guidance stays thin and directs agents to dynamic ctxpack calls instead of injecting large static repository context.
**Plans**: 4 plans
Plans:
- [x] 04-agent-native-client-durability-01-PLAN.md — Add deterministic MCP protocol smoke with explicit-repo wrong-cwd coverage.
- [x] 04-agent-native-client-durability-02-PLAN.md — Keep generated adapter guidance thin, dynamic, and repo-explicit.
- [x] 04-agent-native-client-durability-03-PLAN.md — Make pack resource session scope and cache growth visible/tested.
- [x] 04-agent-native-client-durability-04-PLAN.md — Add optional Codex CLI and Claude Code real-client smoke wrappers.

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
| 1. Compatibility Guardrails & Module Boundaries | 4/4 | Complete | 2026-05-13 |
| 2. Trust Layer & Operational Diagnostics | 5/5 | Complete | 2026-05-13 |
| 3. Measured Retrieval Lift & Eval Gates | 5/5 | Complete | 2026-05-13 |
| 4. Agent-Native Client Durability | 4/4 | Complete | 2026-05-13 |
