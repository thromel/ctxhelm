# Roadmap: Repo Context Packer

## Overview

This roadmap opens v1.6 Repo Memory & Experience Cards. v1.3 made source-free local storage durable, v1.4 added optional local semantic retrieval, and v1.5 added parser/precision graph signals. v1.6 uses those foundations to add durable repo memory that is source-linked, source-free, freshness-aware, human-reviewable, and selectively included in task plans and packs.

## v1.6 Repo Memory & Experience Cards

## Phases

**Phase Numbering:**
- Integer phases (25, 26, 27, 28, 29): Planned v1.6 work
- Decimal phases (26.1, 28.1): Urgent insertions if needed

- [x] **Phase 25: Memory Contracts & Storage Schema** - Source-free memory card contracts and SQLite persistence are stable, versioned, and compatible with existing stores.
- [x] **Phase 26: Freshness-Aware Domain Cards** - Maintainers can generate subsystem/domain cards with source links, input hashes, freshness metadata, and degradation diagnostics.
- [x] **Phase 27: Source-Free Experience Cards** - Maintainers can derive reusable lessons from local traces and explicit structured events without storing raw prompts, logs, or source.
- [x] **Phase 28: Memory Selection in Plans and Packs** - `prepare_task`, `get_pack`, and MCP pack resources select relevant memory under explicit evidence and token-budget caps.
- [x] **Phase 29: Memory Review, Documentation, and Release Gates** - Maintainers can review/redact/disable/regenerate memory, and release gates prove memory safety and usefulness.

## Phase Details

### Phase 25: Memory Contracts & Storage Schema

**Goal**: Source-free memory card contracts and SQLite persistence are stable, versioned, and compatible with existing stores.

**Depends on**: v1.3 Production Storage, v1.4 Local Semantic Retrieval, v1.5 Parser/Semantic Precision

**Requirements**: MEM-01, MEM-02, MEM-03, MEM-04

**Success Criteria** (what must be TRUE):
1. Memory card contracts serialize with stable source-free public JSON shapes.
2. SQLite storage has a versioned memory-card schema with migration history and compatibility diagnostics.
3. Memory storage tests prove raw source, prompt text, terminal logs, secrets, and cloud payloads are not persisted.
4. `ctxpack storage status` reports memory-card counts without breaking older stores.

**Plans**: 1 plan

Plans:
- [x] 25-memory-contracts-storage-schema-01-PLAN.md — Add memory contracts, storage schema, and source-free persistence tests.

### Phase 26: Freshness-Aware Domain Cards

**Goal**: Maintainers can generate subsystem/domain cards with source links, input hashes, freshness metadata, and degradation diagnostics.

**Depends on**: Phase 25

**Requirements**: MEM-05, MEM-06, MEM-07, MEM-08

**Success Criteria** (what must be TRUE):
1. Domain card generation groups safe files into useful subsystem cards using inventory, symbols, tests, dependencies, docs, and existing context-card summaries.
2. Generated cards include source links, input hashes, freshness status, and regeneration reasons.
3. Stale, degraded, or unreviewed cards produce source-free diagnostics instead of silently entering packs.
4. Generation remains deterministic and local-only.

**Plans**: 1 plan

Plans:
- [x] 26-freshness-aware-domain-cards-01-PLAN.md — Extend card generation with domain grouping, freshness, and deterministic diagnostics.

### Phase 27: Source-Free Experience Cards

**Goal**: Maintainers can derive reusable lessons from local traces and explicit structured events without storing raw prompts, logs, or source.

**Depends on**: Phase 26

**Requirements**: MEM-09, MEM-10, MEM-11, MEM-12

**Success Criteria** (what must be TRUE):
1. Experience card ingestion accepts only source-free trace/event metadata and explicit structured corrections.
2. Experience cards contain reusable lessons, source links, provenance, and review status without raw transcript data.
3. Duplicate lessons are merged or reported with source-free provenance.
4. Pending-review behavior prevents unsafe experience cards from pack inclusion by default.

**Plans**: 1 plan

Plans:
- [x] 27-source-free-experience-cards-01-PLAN.md — Add experience card ingestion, dedupe, provenance, and review gating.

### Phase 28: Memory Selection in Plans and Packs

**Goal**: `prepare_task`, `get_pack`, and MCP pack resources select relevant memory under explicit evidence and token-budget caps.

**Depends on**: Phase 27

**Requirements**: MEM-13, MEM-14, MEM-15, MEM-16

**Success Criteria** (what must be TRUE):
1. `prepare_task` returns selected memory candidates with evidence, confidence, source scores, and diagnostics.
2. `get_pack` includes memory in a separate capped section that cannot crowd out target files, tests, or constraints.
3. MCP tools/resources expose memory additively through existing plan/pack flows and stable URI shapes.
4. Historical eval and product proof can compare memory-on and memory-off metadata without source text.

**Plans**: 1 plan

Plans:
- [x] 28-memory-selection-plans-packs-01-PLAN.md — Fuse memory candidates into plans, packs, MCP resources, and eval metadata.

### Phase 29: Memory Review, Documentation, and Release Gates

**Goal**: Maintainers can review/redact/disable/regenerate memory, and release gates prove memory safety and usefulness.

**Depends on**: Phase 28

**Requirements**: MEM-17, MEM-18, MEM-19, MEM-20

**Success Criteria** (what must be TRUE):
1. CLI commands list, show, approve, disable, redact, and regenerate memory cards with Markdown and JSON output.
2. Unsafe generated memory can be rejected before it becomes pack-eligible.
3. Docs explain memory schema, freshness, review workflow, privacy guarantees, MCP behavior, and anti-patterns.
4. Release gate runs deterministic memory smoke coverage for local-only storage, source-free persistence, stale-card diagnostics, selected-memory pack output, and review controls.

**Plans**: 1 plan

Plans:
- [x] 29-memory-review-docs-release-gates-01-PLAN.md — Add review controls, docs, smoke scripts, and release-gate integration.

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| MEM-01 | Phase 25 |
| MEM-02 | Phase 25 |
| MEM-03 | Phase 25 |
| MEM-04 | Phase 25 |
| MEM-05 | Phase 26 |
| MEM-06 | Phase 26 |
| MEM-07 | Phase 26 |
| MEM-08 | Phase 26 |
| MEM-09 | Phase 27 |
| MEM-10 | Phase 27 |
| MEM-11 | Phase 27 |
| MEM-12 | Phase 27 |
| MEM-13 | Phase 28 |
| MEM-14 | Phase 28 |
| MEM-15 | Phase 28 |
| MEM-16 | Phase 28 |
| MEM-17 | Phase 29 |
| MEM-18 | Phase 29 |
| MEM-19 | Phase 29 |
| MEM-20 | Phase 29 |

**Coverage:** 20/20 v1.6 requirements mapped. No orphaned requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 25 -> 26 -> 27 -> 28 -> 29

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 25. Memory Contracts & Storage Schema | 0/1 | Not Started | — |
| 26. Freshness-Aware Domain Cards | 0/1 | Not Started | — |
| 27. Source-Free Experience Cards | 0/1 | Not Started | — |
| 28. Memory Selection in Plans and Packs | 0/1 | Not Started | — |
| 29. Memory Review, Documentation, and Release Gates | 0/1 | Not Started | — |

---
*Roadmap created: 2026-05-16*
