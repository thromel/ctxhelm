# Roadmap: Repo Context Packer

## Overview

This roadmap opens v1.3 Production Storage. v1.2 proved ctxpack's retrieval value with source-free benchmark suites, baseline comparisons, token ROI, gap taxonomy, and product proof reporting. v1.3 turns those repeated-run and scale needs into durable local storage while preserving the read-only, source-free product contract.

## v1.3 Production Storage

## Phases

**Phase Numbering:**
- Integer phases (13, 14, 15, 16): Planned v1.3 work
- Decimal phases (14.1, 14.2): Urgent insertions if needed

- [ ] **Phase 13: Storage Foundation & Schema Contracts** - Maintainers can initialize a versioned SQLite store that captures source-free repository intelligence and migration metadata.
- [ ] **Phase 14: Incremental Indexing & Cache Rebuilds** - Re-indexing large repositories reuses unchanged records, updates stale records, and reports freshness diagnostics.
- [ ] **Phase 15: Evaluation, Pack, and Proof Persistence** - Benchmark, eval, comparison, product-proof, and pack metadata can be persisted and reused without storing source snippets.
- [ ] **Phase 16: Storage Operations, Safety, and Release Gates** - Users can inspect, migrate, repair, clean, and validate storage while preserving privacy and fallback behavior.

## Phase Details

### Phase 13: Storage Foundation & Schema Contracts

**Goal**: Maintainers can initialize a versioned SQLite store that captures source-free repository intelligence and migration metadata.

**Depends on**: v1.2 Retrieval Quality Proof

**Requirements**: STORE-01, STORE-02, STORE-03, STORE-04

**Success Criteria** (what must be TRUE):
1. `ctxpack` can initialize and locate a repo-local or user-local SQLite store with explicit path, schema version, ctxpack version, and privacy metadata.
2. The schema represents repos, files, symbols, chunks, edges, tests, git-history summaries, traces, packs, benchmark runs, and proof reports without raw source text.
3. Storage code has typed contracts and migration/version metadata that are testable without relying on ad hoc JSON files.
4. Privacy tests prove source snippets, prompt text, secrets, and raw file contents are not persisted by default.

**Plans**: 4 plans

Plans:
- [ ] 13-storage-foundation-schema-contracts-01-PLAN.md — Define typed storage contracts, module boundaries, and SQLite store initialization.
- [ ] 13-storage-foundation-schema-contracts-02-PLAN.md — Implement source-free schema tables and metadata records.
- [ ] 13-storage-foundation-schema-contracts-03-PLAN.md — Add schema version, ctxpack version, and migration-history tracking.
- [ ] 13-storage-foundation-schema-contracts-04-PLAN.md — Add privacy tests and fixtures proving source-free persistence defaults.

### Phase 14: Incremental Indexing & Cache Rebuilds

**Goal**: Re-indexing large repositories reuses unchanged records, updates stale records, and reports freshness diagnostics.

**Depends on**: Phase 13

**Requirements**: INCR-01, INCR-02, INCR-03, INCR-04

**Success Criteria** (what must be TRUE):
1. Re-indexing updates changed safe files based on content hash, git blob hash, role classification, ignore policy, and schema version.
2. Search, symbol, test, dependency, history, and card metadata can reuse storage records when source files are unchanged.
3. Stale, missing, corrupted, or policy-incompatible records produce actionable diagnostics instead of silently lowering context quality.
4. Large-repo indexing reports source-free counts for reused, updated, skipped, ignored, generated, and sensitive paths.

**Plans**: 4 plans

Plans:
- [ ] 14-incremental-indexing-cache-rebuilds-01-PLAN.md — Add file fingerprinting and stale-record detection against storage.
- [ ] 14-incremental-indexing-cache-rebuilds-02-PLAN.md — Persist and reuse inventory, symbol, test, dependency, history, and card metadata.
- [ ] 14-incremental-indexing-cache-rebuilds-03-PLAN.md — Add corruption, policy-drift, and partial-store diagnostics.
- [ ] 14-incremental-indexing-cache-rebuilds-04-PLAN.md — Validate incremental behavior on RefactoringMiner-scale fixtures.

### Phase 15: Evaluation, Pack, and Proof Persistence

**Goal**: Benchmark, eval, comparison, product-proof, and pack metadata can be persisted and reused without storing source snippets.

**Depends on**: Phase 14

**Requirements**: PERSIST-01, PERSIST-02, PERSIST-03, PERSIST-04

**Success Criteria** (what must be TRUE):
1. Historical eval, benchmark, comparison, and product-proof runs can be stored with suite, revision, budget, metric, gap, and privacy metadata.
2. Users can compare current benchmark output against stored prior runs without manually passing old artifact paths.
3. Context plan and pack metadata can be stored with task hash, repo snapshot hash, budget, target agent, selected candidate IDs, warnings, and confidence.
4. Stored metadata remains source-free and useful for later semantic retrieval and parser-precision planning.

**Plans**: 4 plans

Plans:
- [ ] 15-evaluation-pack-proof-persistence-01-PLAN.md — Persist source-free eval, benchmark, comparison, and proof run metadata.
- [ ] 15-evaluation-pack-proof-persistence-02-PLAN.md — Add storage-backed benchmark comparison and trend lookup.
- [ ] 15-evaluation-pack-proof-persistence-03-PLAN.md — Persist context plan and pack metadata without snippets by default.
- [ ] 15-evaluation-pack-proof-persistence-04-PLAN.md — Add compatibility tests for future v1.4/v1.5 planning consumers.

### Phase 16: Storage Operations, Safety, and Release Gates

**Goal**: Users can inspect, migrate, repair, clean, and validate storage while preserving privacy and fallback behavior.

**Depends on**: Phase 15

**Requirements**: OPS-01, OPS-02, OPS-03, OPS-04

**Success Criteria** (what must be TRUE):
1. CLI exposes storage status, migration, repair, vacuum/cleanup, and reset commands with dry-run or confirmation behavior for destructive actions.
2. MCP and CLI diagnostics include storage freshness, migration status, privacy status, and degradation warnings when results depend on stale or partial storage.
3. Release/adoption gates verify schema compatibility, migration behavior, source-free storage guarantees, and fallback behavior when storage is unavailable.
4. Documentation explains storage location, privacy guarantees, repair/reset flows, and repeated-workflow benefits.

**Plans**: 4 plans

Plans:
- [ ] 16-storage-operations-safety-release-gates-01-PLAN.md — Add storage status, migrate, repair, cleanup, and reset CLI commands.
- [ ] 16-storage-operations-safety-release-gates-02-PLAN.md — Surface storage diagnostics through CLI and MCP outputs.
- [ ] 16-storage-operations-safety-release-gates-03-PLAN.md — Extend release gates for schema, migration, privacy, and fallback checks.
- [ ] 16-storage-operations-safety-release-gates-04-PLAN.md — Document storage behavior, privacy guarantees, and recovery workflows.

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| STORE-01 | Phase 13 |
| STORE-02 | Phase 13 |
| STORE-03 | Phase 13 |
| STORE-04 | Phase 13 |
| INCR-01 | Phase 14 |
| INCR-02 | Phase 14 |
| INCR-03 | Phase 14 |
| INCR-04 | Phase 14 |
| PERSIST-01 | Phase 15 |
| PERSIST-02 | Phase 15 |
| PERSIST-03 | Phase 15 |
| PERSIST-04 | Phase 15 |
| OPS-01 | Phase 16 |
| OPS-02 | Phase 16 |
| OPS-03 | Phase 16 |
| OPS-04 | Phase 16 |

**Coverage:** 16/16 v1.3 requirements mapped. No orphaned requirements.

## Progress

**Execution Order:**
Phases execute in numeric order: 13 -> 14 -> 15 -> 16

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 13. Storage Foundation & Schema Contracts | 0/4 | Not started | — |
| 14. Incremental Indexing & Cache Rebuilds | 0/4 | Blocked on Phase 13 | — |
| 15. Evaluation, Pack, and Proof Persistence | 0/4 | Blocked on Phase 14 | — |
| 16. Storage Operations, Safety, and Release Gates | 0/4 | Blocked on Phase 15 | — |
