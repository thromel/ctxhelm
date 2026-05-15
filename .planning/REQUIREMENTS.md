# Requirements: Repo Context Packer v1.4 Local Semantic Retrieval

**Defined:** 2026-05-16
**Milestone:** v1.4 Local Semantic Retrieval
**Core Value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

## v1.4 Requirements

Requirements for adding optional local embedding/vector retrieval as a measured signal inside the existing context compiler.

### Local Semantic Foundation

- [x] **SEM-01**: Maintainer can enable or disable local semantic retrieval per repository or invocation, and the default remains disabled until a local provider is configured.
- [x] **SEM-02**: Maintainer can configure a local embedding provider through a typed provider interface that records provider name, model/version, dimensions, distance metric, and privacy status.
- [x] **SEM-03**: Semantic index metadata and vectors are stored locally with explicit privacy labels and without persisting raw source snippets, prompt text, secrets, or cloud request payloads.
- [x] **SEM-04**: Semantic indexing only processes files and chunks allowed by the safe inventory policy, including ignore, generated-file, sensitive-file, and source-read revalidation rules.

### Vector Candidate Retrieval

- [x] **SEM-05**: Agent or CLI user can request semantic candidate generation for conceptual tasks and receive vector candidates with stable IDs, scores, source attribution, and human-readable reasons.
- [x] **SEM-06**: Re-indexing reuses unchanged semantic records and only refreshes vectors when safe chunk hashes, provider configuration, model version, or privacy policy changes.
- [x] **SEM-07**: CLI and MCP surfaces expose semantic retrieval through existing task/search/pack workflows without adding a broad new MCP tool surface.
- [x] **SEM-08**: Missing, disabled, incompatible, or failed local embedding providers degrade to lexical/graph/history/test retrieval with explicit non-fatal diagnostics.

### Hybrid Fusion And Pack Behavior

- [x] **SEM-09**: Context planning fuses vector candidates with lexical, symbol, graph, test, history, and active-context signals using task-specific weights and exact-match safeguards.
- [x] **SEM-10**: Context plans, packs, eval traces, and MCP responses expose semantic signal provenance and privacy status without logging source text by default.
- [x] **SEM-11**: Budget allocation and diversification prevent vector near-duplicates from crowding out target files, related tests, direct dependencies, or exact identifier matches.
- [x] **SEM-12**: Existing CLI JSON and MCP contracts remain backward compatible, with additive semantic fields covered by compatibility tests and source-free snapshots.

### Semantic Evaluation And Release Proof

- [x] **SEM-13**: Historical eval and benchmark suites can compare semantic-enabled retrieval against lexical/graph/history/test baselines at fixed budgets.
- [x] **SEM-14**: Benchmark comparison and product-proof reports show semantic lift, regressions, token ROI, missing-file gaps, and privacy status in source-free output.
- [x] **SEM-15**: Release/adoption gates include deterministic local semantic smoke coverage when a fixture provider is available and prove cloud embeddings/reranking stay disabled by default.
- [x] **SEM-16**: Documentation explains local semantic setup, provider configuration, privacy boundaries, failure modes, reset/repair behavior, and when semantic retrieval should be avoided.

## Future Requirements

Deferred to future milestones from the original product vision and refined by v1.2-v1.4 evidence.

### v1.5 Parser/Semantic Precision

- **PARS-04**: Expand Tree-sitter-backed language coverage where benchmark gaps justify it.
- **PARS-05**: Add optional SCIP/LSP ingestion for precise definitions, references, implementations, and call edges.
- **PARS-06**: Prove precision lift without recursive graph/context explosion.
- **PARS-07**: Degrade safely when language tooling or project setup is unavailable.

### v1.6 Repo Memory And Experience Cards

- **MEM-01**: Generate domain cards for important subsystems with freshness metadata, source links, and regeneration triggers.
- **MEM-02**: Store source-free experience cards from prior agent sessions, test failures, accepted fixes, and user corrections.
- **MEM-03**: Select relevant repo memory during `prepare_task` and `get_pack` with explicit evidence, confidence, and token-budget caps.
- **MEM-04**: Provide human review, redaction, and disable controls so generated memory stays trustworthy and editable.

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

v1 through v1.4 are complete and archived or ready to archive. They validated:

- CLI, MCP, and public JSON compatibility guardrails.
- Safe inventory, diagnostics, privacy/source-read policy, and source-free local traces.
- Typed attributed retrieval candidates, context plans, context packs, historical evals, signal ablations, and gap reports.
- Codex CLI and Claude Code MCP smoke proof with explicit repo arguments.
- v1.1 binary packaging, checksums, artifact audit, install docs, setup validation, first-pack smoke, and release gates.
- v1.2 named benchmark suites, fixed-budget baseline comparisons, token ROI, gap taxonomy, benchmark comparison, and product proof reporting.
- v1.3 durable source-free SQLite storage, incremental indexing metadata, pack/eval/proof persistence, storage operations, and storage release gates.
- v1.4 optional local semantic retrieval, source-free vector metadata, semantic fusion, semantic eval flags, documentation, and release-gate smoke coverage.

## Out of Scope

Explicitly excluded from v1.4 to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Cloud embeddings, hosted vector search, or cloud reranking by default | v1.4 must preserve local-first trust; any cloud path remains future opt-in policy work. |
| Autonomous code editing | ctxpack remains a read-only context broker; agents own edits and permissions. |
| SCIP/LSP precision integration | Parser precision belongs in v1.5 after semantic retrieval produces measured gaps. |
| Broad parser or Tree-sitter language expansion | v1.4 focuses on semantic retrieval as a retrieval signal, not syntax precision. |
| New standalone daily app or UI | Agent-native CLI/MCP/rules remain the product surface; diagnostics UI belongs in v2.1. |
| Storing raw source snippets, prompt text, or cloud request payloads by default | Source-free persistence remains the trust contract for evals, packs, and proof. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SEM-01 | Phase 17 | Complete |
| SEM-02 | Phase 17 | Complete |
| SEM-03 | Phase 17 | Complete |
| SEM-04 | Phase 17 | Complete |
| SEM-05 | Phase 18 | Complete |
| SEM-06 | Phase 18 | Complete |
| SEM-07 | Phase 18 | Complete |
| SEM-08 | Phase 18 | Complete |
| SEM-09 | Phase 19 | Complete |
| SEM-10 | Phase 19 | Complete |
| SEM-11 | Phase 19 | Complete |
| SEM-12 | Phase 19 | Complete |
| SEM-13 | Phase 20 | Complete |
| SEM-14 | Phase 20 | Complete |
| SEM-15 | Phase 20 | Complete |
| SEM-16 | Phase 20 | Complete |

**Coverage:**
- v1.4 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0

---
*Requirements defined: 2026-05-16*
