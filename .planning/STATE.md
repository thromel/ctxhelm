---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: complete
stopped_at: Completed 04-agent-native-client-durability-04-PLAN.md
last_updated: "2026-05-13T17:06:11Z"
last_activity: 2026-05-13
progress:
  total_phases: 4
  completed_phases: 4
  total_plans: 18
  completed_plans: 18
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-13)

**Core value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.
**Current focus:** Milestone complete — all v1 phases verified

## Current Position

Phase: 4
Plan: 04
Status: Complete — all 18 plans complete
Last activity: 2026-05-13

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 0
- Average duration: N/A
- Total execution time: 0.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 5 plans: none
- Trend: N/A

*Updated after each plan completion*
| Phase 01-compatibility-guardrails-module-boundaries P01 | 6 min | 2 tasks | 4 files |
| Phase 01-compatibility-guardrails-module-boundaries P02 | 5m34s | 2 tasks | 3 files |
| Phase 01-compatibility-guardrails-module-boundaries P03 | 9m7s | 3 tasks | 8 files |
| Phase 01-compatibility-guardrails-module-boundaries P04 | 8m10s | 3 tasks | 11 files |
| Phase 02-trust-layer-operational-diagnostics P01 | 6m52s | 2 tasks | 6 files |
| Phase 02-trust-layer-operational-diagnostics P02 | 7m49s | 2 tasks | 4 files |
| Phase 02-trust-layer-operational-diagnostics P03 | 13m34s | 2 tasks | 8 files |
| Phase 02-trust-layer-operational-diagnostics P04 | 9m47s | 2 tasks | 6 files |
| Phase 02-trust-layer-operational-diagnostics P05 | 9m57s | 3 tasks | 8 files |
| Phase 03-measured-retrieval-lift-eval-gates P01 | 10m05s | 2 tasks | 3 files |
| Phase 03-measured-retrieval-lift-eval-gates P02 | 11m37s | 2 tasks | 4 files |
| Phase 03-measured-retrieval-lift-eval-gates P03 | 8m49s | 2 tasks | 5 files |
| Phase 03-measured-retrieval-lift-eval-gates P04 | 9m56s | 3 tasks | 3 files |
| Phase 03-measured-retrieval-lift-eval-gates P05 | 4m | 3 tasks | 3 files |
| Phase 04-agent-native-client-durability P02 | 3m | 2 tasks | 2 files |
| Phase 04-agent-native-client-durability P01 | 5m15s | 2 tasks | 2 files |
| Phase 04-agent-native-client-durability P03 | 4m20s | 2 tasks | 4 files |
| Phase 04-agent-native-client-durability P04 | 6m11s | 2 tasks | 3 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: Use coarse granularity with 4 broad phases derived from current requirements.
- [Roadmap]: Protect public CLI, MCP, and JSON contracts before module splits and retrieval changes.
- [Roadmap]: Treat freshness, privacy, safe source reads, and diagnostics as prerequisites for measured retrieval lift.
- [Roadmap]: Keep parser/runtime upgrades gated by eval evidence rather than broad migration.
- [Phase 01-compatibility-guardrails-module-boundaries]: Plan 01 uses binary-level assert_cmd tests with real temp repos and command-local CTXPACK_HOME to guard CLI compatibility.
- [Phase 01-compatibility-guardrails-module-boundaries]: Plan 02 guards public JSON compatibility with explicit serde_json field-shape assertions rather than schema generation.
- [Phase 01-compatibility-guardrails-module-boundaries]: Plan 02 characterizes MCP pack resources as session-scoped cache entries without adding cross-process durability.
- [Phase 01-compatibility-guardrails-module-boundaries]: Plan 03 kept ctxpack-index crate root as the public facade while moving implementation into focused private modules.
- [Phase 01-compatibility-guardrails-module-boundaries]: Plan 03 kept shared index helpers crate-visible instead of public to preserve API boundaries.
- [Phase 01-compatibility-guardrails-module-boundaries]: Plan 04 kept ctxpack-compiler and ctxpack-mcp crate roots as stable public facades while moving implementation into concern-focused modules.
- [Phase 01-compatibility-guardrails-module-boundaries]: Plan 04 used pub(crate) only for shared internals needed across the new module boundaries and existing crate-local tests.
- [Phase 02-trust-layer-operational-diagnostics]: Diagnostics are additive serde fields on ContextPlan and ContextPack, with riskFlags preserved unchanged for compatibility.
- [Phase 02-trust-layer-operational-diagnostics]: ctxpack-index policy.rs owns path role classification and safe source-read outcomes, while lib.rs remains the public facade.
- [Phase 02-trust-layer-operational-diagnostics]: Safe source reads return typed SourceRead values with source-free diagnostics instead of CLI/MCP formatting strings.
- [Phase 02-trust-layer-operational-diagnostics]: Inventory metadata is additive on RepoInventory and missing metadata deserializes as a stale legacy cache.
- [Phase 02-trust-layer-operational-diagnostics]: Freshness diagnostics are source-free and report reason codes, paths, and counts without snippets.
- [Phase 02-trust-layer-operational-diagnostics]: load_or_refresh_inventory returns fresh in-memory inventory when cache persistence fails, while write_inventory remains fatal.
- [Phase 02-trust-layer-operational-diagnostics]: Index read APIs keep existing result shapes while exposing diagnostic report variants for downstream compiler and MCP wiring.
- [Phase 02-trust-layer-operational-diagnostics]: Search, symbols, related tests, and dependency graph parse only content returned by read_safe_source.
- [Phase 02-trust-layer-operational-diagnostics]: New git report APIs convert missing or timed-out git into diagnostics, while legacy co_change_hints still errors for existing compiler risk-flag compatibility.
- [Phase 02-trust-layer-operational-diagnostics]: Compiler planning consumes diagnostic report APIs and mirrors warning/error diagnostics into riskFlags for compatibility.
- [Phase 02-trust-layer-operational-diagnostics]: Pack compilation keeps existing public APIs while representing revalidation failures as ContextPack diagnostics and warnings.
- [Phase 02-trust-layer-operational-diagnostics]: Context card reports now include additive diagnostics while generated cards remain source-free.
- [Phase 02-trust-layer-operational-diagnostics]: Trace recording remains on by default, with additive CLI --no-trace and MCP recordTrace controls for read-oriented commands.
- [Phase 02-trust-layer-operational-diagnostics]: MCP related_tests preserves its existing array-shaped structuredContent and exposes diagnostics at the tool-result level to avoid a breaking shape change.
- [Phase 02-trust-layer-operational-diagnostics]: MCP file resources now revalidate source paths against fresh safe inventory before reading source text.
- [Phase 03-measured-retrieval-lift-eval-gates]: Add retrievalCandidates and attribution as additive serde fields with default empty vectors to preserve old JSON compatibility.
- [Phase 03-measured-retrieval-lift-eval-gates]: Expose typed candidate/evidence contracts now, but keep all ranking, pack, MCP, and eval behavior unchanged in Plan 01.
- [Phase 03-measured-retrieval-lift-eval-gates]: Keep attribution source-free by omitting task text, source snippets, symbol signatures, and commit subject fields.
- [Phase 03-measured-retrieval-lift-eval-gates]: Keep ranking private to ctxpack-compiler while projecting through existing ContextPlan fields for compatibility.
- [Phase 03-measured-retrieval-lift-eval-gates]: Treat explicit anchors and current-diff anchors as high-priority signals so active context remains first under fixed budgets.
- [Phase 03-measured-retrieval-lift-eval-gates]: Infer current-diff attribution from safe changed paths without adding a new public planning parameter.
- [Phase 03-measured-retrieval-lift-eval-gates]: Preserve safeChangedFiles as the compatibility projection while making changedPaths/changedPathLabels the rich source-free label surface.
- [Phase 03-measured-retrieval-lift-eval-gates]: Keep HistoricalEvalOptions source-compatible in Plan 03 and freeze the current Standard budget in report metadata.
- [Phase 03-measured-retrieval-lift-eval-gates]: Use parent-snapshot worktrees for replay and source-free path labels for truth, avoiding source snippets and commit subjects in serialized reports.
- [Phase 03-measured-retrieval-lift-eval-gates]: Keep legacy Recall@5/10 fields while adding rankingComparison as the fixed-budget decision-grade eval surface.
- [Phase 03-measured-retrieval-lift-eval-gates]: Use typed source-free SignalAblationResult and RetrievalGapSummary records for eval history and checklist reporting.
- [Phase 03-measured-retrieval-lift-eval-gates]: Keep parser/runtime upgrades gated by measured gaps and add no Tantivy, rayon, SQLite, notify, tree-sitter, or MCP SDK dependency in Plan 04.
- [Phase 03-measured-retrieval-lift-eval-gates]: Keep Plan 05 validation additive: tests assert new Phase 3 fields while preserving existing CLI/MCP keys and tool names.
- [Phase 03-measured-retrieval-lift-eval-gates]: Use a source-free smoke script that validates fixed-budget report metadata instead of adding new runtime behavior.
- [Phase 03-measured-retrieval-lift-eval-gates]: Represent the validation-only task with an empty task commit so the GSD per-task commit trail remains complete.
- [Phase 04-agent-native-client-durability]: Keep generated adapter text as concise runtime guidance rather than static repository context.
- [Phase 04-agent-native-client-durability]: Tell agents to call prepare_task first with explicit repo, then request get_pack progressively only when direct file reads or brief context are insufficient.
- [Phase 04-agent-native-client-durability]: Plan 01 uses deterministic JSON-RPC stdio as the Phase 4 hard gate before optional real-client smokes.
- [Phase 04-agent-native-client-durability]: Plan 01 requires all repo-accepting MCP smoke calls to pass explicit repo from a server cwd outside the target repo.
- [Phase 04-agent-native-client-durability]: Plan 01 reads pack resources using prepare_task-returned task-scoped URIs instead of fixed pack URI names.
- [Phase 04-agent-native-client-durability]: Preserve process-local MCP pack resources and make the session boundary explicit instead of adding persistence or reconstruction.
- [Phase 04-agent-native-client-durability]: Use get_pack as the durable reconnect-safe materialization path after MCP client reconnects or server restarts.
- [Phase 04-agent-native-client-durability]: Bound pack resource cache growth privately with deterministic oldest-key eviction and test-only inspection helpers.
- [Phase 04-agent-native-client-durability]: Keep deterministic protocol smoke as the hard gate before any Codex or Claude attempt.
- [Phase 04-agent-native-client-durability]: Use isolated temp state for CTXPACK_HOME, Codex execution, Claude MCP config, and server-side request logs.
- [Phase 04-agent-native-client-durability]: Require machine-checkable prepare_task and get_pack calls with the explicit repo; final assistant prose is not proof.

### Pending Todos

None yet.

### Blockers/Concerns

None currently.

## Session Continuity

Last session: 2026-05-13T17:06:11Z
Stopped at: Verified Phase 04-agent-native-client-durability
Resume file: None
