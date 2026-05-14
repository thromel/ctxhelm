# Requirements: Repo Context Packer v1.3 Production Storage

**Defined:** 2026-05-14
**Milestone:** v1.3 Production Storage
**Core Value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

## v1.3 Requirements

Requirements for replacing ad hoc JSON/cache behavior with durable, fast, local, source-free storage.

### Storage Foundation

- [x] **STORE-01**: Maintainer can initialize a repo-local or user-local SQLite store with explicit path, version, and privacy metadata.
- [x] **STORE-02**: Store schema captures source-free repository metadata for repos, files, symbols, chunks, dependency edges, tests, git history summaries, traces, packs, benchmark runs, and proof reports.
- [x] **STORE-03**: Store records schema version, ctxpack version, ranking/compiler version, and migration history so stale or incompatible storage can be diagnosed.
- [x] **STORE-04**: Store defaults never persist source snippets, prompt text, secrets, or raw file contents; any future source-bearing storage must be explicit and privacy-labeled.

### Incremental Indexing

- [ ] **INCR-01**: Maintainer can re-index a repository and update only changed safe files when content hashes, git blob hashes, role classification, or ignore policy changed.
- [ ] **INCR-02**: Search, symbol, test, dependency, history, and card metadata can be rebuilt from the durable store without reparsing unchanged files.
- [ ] **INCR-03**: ctxpack reports stale, missing, corrupted, or policy-incompatible store records with actionable diagnostics instead of silently returning low-confidence context.
- [ ] **INCR-04**: Large-repo indexing reports source-free counts for reused records, updated records, skipped files, ignored paths, generated files, and sensitive exclusions.

### Evaluation And Pack Persistence

- [ ] **PERSIST-01**: Historical eval, benchmark, comparison, and product-proof runs can be persisted as source-free records with suite, revision, budget, metric, gap, and privacy metadata.
- [ ] **PERSIST-02**: Maintainer can compare current benchmark output against stored prior runs without manually managing JSON artifact paths.
- [ ] **PERSIST-03**: Context plan and pack metadata can be persisted with task hash, repo snapshot hash, budget, target agent, selected candidate IDs, warnings, and confidence without storing snippets by default.
- [ ] **PERSIST-04**: Stored benchmark and pack metadata remains usable for future v1.4/v1.5 planning without requiring access to original source snippets.

### Operations And Safety

- [ ] **OPS-01**: CLI exposes storage status, migration, repair, vacuum/cleanup, and reset commands with dry-run or confirmation behavior for destructive actions.
- [ ] **OPS-02**: MCP and CLI diagnostics include storage freshness, migration status, privacy status, and degradation warnings when context results depend on stale or partial storage.
- [ ] **OPS-03**: Release/adoption gates can verify schema compatibility, migration behavior, source-free storage guarantees, and fallback behavior when the store is unavailable.
- [ ] **OPS-04**: Documentation explains storage location, privacy guarantees, repair/reset flows, and how storage improves repeated agent workflows without becoming a cloud sync layer.

## Future Requirements

Deferred to future milestones from the original product vision and refined by v1.2-v1.3 evidence.

### v1.4 Local Semantic Retrieval

- **SEM-01**: Add an optional local embedding provider interface with explicit privacy status.
- **SEM-02**: Fuse vector candidates with lexical, symbol, graph, test, history, and active-context signals.
- **SEM-03**: Prove semantic lift through fixed-budget evals before making semantic retrieval prominent.
- **SEM-04**: Keep cloud embeddings and reranking opt-in, visibly labeled, and disabled by default.

### v1.5 Parser/Semantic Precision

- **PARS-04**: Expand Tree-sitter-backed language coverage where benchmark gaps justify it.
- **PARS-05**: Add optional SCIP/LSP ingestion for precise definitions, references, implementations, and call edges.
- **PARS-06**: Prove precision lift without recursive graph/context explosion.
- **PARS-07**: Degrade safely when language tooling or project setup is unavailable.

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

v1, v1.1, and v1.2 are complete and archived. They validated:

- CLI, MCP, and public JSON compatibility guardrails.
- Safe inventory, diagnostics, privacy/source-read policy, and source-free local traces.
- Typed attributed retrieval candidates, context plans, context packs, historical evals, signal ablations, and gap reports.
- Codex CLI and Claude Code MCP smoke proof with explicit repo arguments.
- v1.1 binary packaging, checksums, artifact audit, install docs, setup validation, first-pack smoke, and release gates.
- v1.2 named benchmark suites, fixed-budget baseline comparisons, token ROI, gap taxonomy, benchmark comparison, and product proof reporting.

## Out of Scope

Explicitly excluded from v1.3 to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Embeddings or vector retrieval | Semantic retrieval belongs in v1.4 after durable storage exists. |
| SCIP/LSP precision integration | Parser precision belongs in v1.5 and should follow measured storage/retrieval gaps. |
| Hosted sync, remote database, or team backend | v1.3 must stay local-first and source-free. |
| Autonomous code editing | ctxpack remains a read-only context broker; agents own edits and permissions. |
| Storing raw source snippets by default | Source-free persistence is the trust contract for evals, packs, and proof. |
| Full UI or pack inspector | UI belongs in v2.1 after storage-backed diagnostics exist. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| STORE-01 | Phase 13 | Complete |
| STORE-02 | Phase 13 | Complete |
| STORE-03 | Phase 13 | Complete |
| STORE-04 | Phase 13 | Complete |
| INCR-01 | Phase 14 | Planned |
| INCR-02 | Phase 14 | Planned |
| INCR-03 | Phase 14 | Planned |
| INCR-04 | Phase 14 | Planned |
| PERSIST-01 | Phase 15 | Planned |
| PERSIST-02 | Phase 15 | Planned |
| PERSIST-03 | Phase 15 | Planned |
| PERSIST-04 | Phase 15 | Planned |
| OPS-01 | Phase 16 | Planned |
| OPS-02 | Phase 16 | Planned |
| OPS-03 | Phase 16 | Planned |
| OPS-04 | Phase 16 | Planned |

**Coverage:**
- v1.3 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0

---
*Requirements defined: 2026-05-14*
