# Requirements: Repo Context Packer v1.6 Repo Memory & Experience Cards

**Defined:** 2026-05-16
**Milestone:** v1.6 Repo Memory & Experience Cards
**Core Value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

## v1.6 Requirements

Requirements for durable, source-linked, source-free memory that agents can reuse selectively without bloating every pack.

### Memory Contracts and Storage

- [x] **MEM-01**: Maintainer can store source-free memory card metadata in local SQLite with schema versioning, migration history, privacy labels, timestamps, and compatibility diagnostics.
- [x] **MEM-02**: Maintainer can represent domain cards and experience cards through stable typed contracts with IDs, kinds, titles, summaries, source links, input hashes, freshness status, review status, and disable status.
- [x] **MEM-03**: Memory storage excludes raw source snippets, prompt text, terminal logs, secrets, cloud payloads, and unredacted user freeform text by default.
- [x] **MEM-04**: Existing storage commands report memory-card counts and compatibility without breaking older stores.

### Domain Cards

- [x] **MEM-05**: Maintainer can generate domain cards for important subsystems using safe inventory, symbols, tests, dependency edges, docs, and existing context-card summaries.
- [x] **MEM-06**: Generated domain cards include source links, input hashes, freshness metadata, and regeneration reasons.
- [x] **MEM-07**: Stale, degraded, or unreviewed domain cards are surfaced with source-free diagnostics instead of silently entering packs.
- [x] **MEM-08**: Domain card generation remains deterministic and local-only unless a future explicit provider is added.

### Experience Cards

- [x] **MEM-09**: Maintainer can derive source-free experience card candidates from local eval traces, pack metadata, test failures recorded as structured events, accepted fixes, and explicit user corrections.
- [x] **MEM-10**: Experience cards capture reusable lessons as source-linked summaries with evidence paths and event metadata, not raw transcripts.
- [x] **MEM-11**: Experience card ingestion deduplicates repeated lessons and records provenance so humans can audit why a lesson exists.
- [x] **MEM-12**: Experience cards default to pending review before pack inclusion unless created from deterministic source-free events.

### Memory Selection

- [x] **MEM-13**: `prepare_task` can select relevant memory cards as a bounded retrieval signal with explicit evidence, confidence, source scores, and diagnostics.
- [x] **MEM-14**: `get_pack` can include selected memory under a separate token budget cap without crowding out target files, tests, or critical constraints.
- [x] **MEM-15**: MCP tools and resources expose selected memory additively through existing plan/pack flows and stable URI shapes without adding a large new tool surface.
- [x] **MEM-16**: Historical eval and product proof can report memory-enabled metadata and compare memory-on versus memory-off behavior without storing source text.

### Review, Redaction, Docs, and Gates

- [x] **MEM-17**: Maintainer can list, show, approve, disable, and regenerate memory cards through CLI commands with JSON and Markdown output.
- [x] **MEM-18**: Maintainer can redact or reject unsafe generated memory before it becomes pack-eligible.
- [x] **MEM-19**: Documentation explains memory card schema, freshness, review workflow, privacy guarantees, MCP behavior, and when memory should not be used.
- [x] **MEM-20**: Release gate includes deterministic memory smoke coverage proving local-only storage, source-free persistence, stale-card diagnostics, selected-memory pack output, and review controls.

## Future Requirements

Deferred to future milestones from the original product vision and refined by v1.2-v1.6 evidence.

### v1.7 Adaptive Retrieval Policy And Feedback Loop

- **LEARN-01**: Ingest source-free session feedback for recommended, read, edited, tested, passed, failed, and user-corrected files.
- **LEARN-02**: Report policy-level context precision, signal weight contribution, token ROI, and repeated missing-file families.
- **LEARN-03**: Tune retrieval policy from benchmark and session evidence with rollback when a signal regresses.
- **LEARN-04**: Compare agent outcomes across plan-only, brief, standard, and deep packs on fixed tasks.

### v2.0 Workspace And Team Layer

- **TEAM-01**: Support multi-repo workspace inventory and task context planning.
- **TEAM-02**: Support source-free shared context cards, benchmark reports, and policy files.
- **TEAM-03**: Add team-level privacy policy templates without hosted source indexing.
- **TEAM-04**: Keep agent-native MCP/rules surfaces as the primary workflow.

### v2.1 UI / Pack Inspector

- **UI-01**: Add an optional pack inspector that shows target files, evidence, token budgets, omitted candidates, and warnings.
- **UI-02**: Add retrieval-health views for benchmark trends and repeated gap families.
- **UI-03**: Add context-card and adapter preview views.
- **UI-04**: Keep the UI diagnostic; daily coding remains inside existing agents.

## Completed Requirements

v1 through v1.5 are complete. They validated:

- CLI, MCP, and public JSON compatibility guardrails.
- Safe inventory, diagnostics, privacy/source-read policy, and source-free local traces.
- Typed attributed retrieval candidates, context plans, context packs, historical evals, signal ablations, and gap reports.
- Codex CLI and Claude Code MCP smoke proof with explicit repo arguments.
- v1.1 binary packaging, checksums, artifact audit, install docs, setup validation, first-pack smoke, and release gates.
- v1.2 named benchmark suites, fixed-budget baseline comparisons, token ROI, gap taxonomy, benchmark comparison, and product proof reporting.
- v1.3 durable source-free SQLite storage, incremental indexing metadata, pack/eval/proof persistence, storage operations, and storage release gates.
- v1.4 optional local semantic retrieval, source-free vector metadata, semantic fusion, semantic eval flags, documentation, and release-gate smoke coverage.
- v1.5 Java/Kotlin symbol and dependency precision plus source-free SCIP/LSP bridge edge import.

## Out of Scope

Explicitly excluded from v1.6 to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Adaptive retrieval-policy tuning | v1.7 owns feedback-driven policy changes and rollback. v1.6 only surfaces memory as a bounded signal. |
| Cloud memory generation | v1.6 preserves local-first trust and source-free persistence. |
| Raw chat transcript ingestion | Prompt text and freeform session logs are too privacy-sensitive for default memory storage. |
| Always-injected memory | Memory must be selected by task and budgeted; dumping all cards harms context quality. |
| Team/shared memory sync | v2.0 owns team and workspace sharing. |
| UI pack inspector | v2.1 owns diagnostics UI. |
| Autonomous code editing | ctxpack remains a read-only context broker; agents own edits and permissions. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| MEM-01 | Phase 25 | Planned |
| MEM-02 | Phase 25 | Planned |
| MEM-03 | Phase 25 | Planned |
| MEM-04 | Phase 25 | Planned |
| MEM-05 | Phase 26 | Planned |
| MEM-06 | Phase 26 | Planned |
| MEM-07 | Phase 26 | Planned |
| MEM-08 | Phase 26 | Planned |
| MEM-09 | Phase 27 | Planned |
| MEM-10 | Phase 27 | Planned |
| MEM-11 | Phase 27 | Planned |
| MEM-12 | Phase 27 | Planned |
| MEM-13 | Phase 28 | Planned |
| MEM-14 | Phase 28 | Planned |
| MEM-15 | Phase 28 | Planned |
| MEM-16 | Phase 28 | Planned |
| MEM-17 | Phase 29 | Planned |
| MEM-18 | Phase 29 | Planned |
| MEM-19 | Phase 29 | Planned |
| MEM-20 | Phase 29 | Planned |
| LEARN-01 | Future v1.7 | Deferred |
| LEARN-02 | Future v1.7 | Deferred |
| LEARN-03 | Future v1.7 | Deferred |
| LEARN-04 | Future v1.7 | Deferred |
| TEAM-01 | Future v2.0 | Deferred |
| TEAM-02 | Future v2.0 | Deferred |
| TEAM-03 | Future v2.0 | Deferred |
| TEAM-04 | Future v2.0 | Deferred |
| UI-01 | Future v2.1 | Deferred |
| UI-02 | Future v2.1 | Deferred |
| UI-03 | Future v2.1 | Deferred |
| UI-04 | Future v2.1 | Deferred |

**Coverage:**
- v1.6 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0

---
*Requirements defined: 2026-05-16*
