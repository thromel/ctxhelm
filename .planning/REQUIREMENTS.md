# Requirements: Repo Context Packer v1.2 Retrieval Quality Proof

**Defined:** 2026-05-14
**Milestone:** v1.2 Retrieval Quality Proof
**Core Value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

## v1.2 Requirements

Requirements for proving ctxpack's product value through repeatable, source-free real-repo retrieval benchmarks.

### Benchmark Corpus

- [ ] **BENCH-01**: Maintainer can define named benchmark suites with repo path, revision range, max commits, retrieval budgets, and role filters.
- [ ] **BENCH-02**: Maintainer can run bounded historical evals over RefactoringMiner and at least one additional real repository using reproducible revisions.
- [ ] **BENCH-03**: Benchmark artifacts include source-free labels for changed files, related tests, skipped paths, privacy status, and run metadata.
- [ ] **BENCH-04**: Maintainer has documentation for adding another local real-repo benchmark without cloud services or global machine assumptions.

### Retrieval Metrics And Baselines

- [ ] **METR-01**: Benchmark reports include file Recall@K, test Recall@K, useful-target ratios, and missing-label summaries for each suite.
- [ ] **METR-02**: Benchmark reports compare ctxpack hybrid ranking against lexical-only and no-context or anchor-only baselines under fixed budgets.
- [ ] **METR-03**: Benchmark reports include signal ablations for symbols, dependencies, tests, git history, current diff, and docs/cards.
- [ ] **METR-04**: Benchmark reports are available as stable JSON and human-readable Markdown without source snippets or prompt text.
- [ ] **METR-05**: Benchmark reports include enough metadata to reproduce the same suite, budget, and revision selection.

### Token ROI

- [ ] **ROI-01**: Benchmark reports estimate useful targets per 1k context tokens for brief, standard, and deep budget options.
- [ ] **ROI-02**: Benchmark reports identify when larger packs add little or negative value compared with smaller plans or brief packs.

### Gap Analysis

- [ ] **GAP-01**: Repeated missing files and tests are grouped into source-free families by role, path pattern, package, status, and missing signal.
- [ ] **GAP-02**: Gap reports identify whether failures point to storage, semantic retrieval, parser precision, test mapping, history ranking, or policy exclusions.
- [ ] **GAP-03**: Gap reports distinguish deleted/renamed historical labels from current reachable targets so evals do not punish impossible retrieval.
- [ ] **GAP-04**: Gap reports can be consumed by future milestone planning without needing access to source snippets.

### Regression Trends

- [ ] **REG-01**: Maintainer can compare two benchmark runs and see deltas for recall, token ROI, signal ablations, skipped files, and gap families.
- [ ] **REG-02**: Maintainer can configure threshold checks that fail when selected retrieval metrics regress beyond allowed tolerances.

### Product Proof

- [ ] **PROOF-01**: README or docs include a concise source-free proof report explaining benchmark setup, headline metrics, baseline deltas, and limitations.
- [ ] **PROOF-02**: `ctxpack eval` exposes a maintainer-friendly command path that reproduces the proof report on configured local repos.
- [ ] **PROOF-03**: Release or adoption gates can optionally run a bounded benchmark smoke and fail on report-generation or privacy regressions.
- [ ] **PROOF-04**: Product documentation clearly states when ctxpack helps, when it does not, and how agents should use the evidence.
- [ ] **PROOF-05**: Future milestone requirements for v1.3-v2.1 are updated from measured retrieval gaps rather than speculative feature desire.

## Future Requirements

Deferred to future milestones from the original product vision.

### v1.3 Production Storage

- **STORE-01**: Add SQLite-backed local storage for repository metadata, symbols, chunks, edges, traces, packs, and benchmark results.
- **STORE-02**: Add faster incremental indexing and cache invalidation for large repositories.
- **STORE-03**: Add schema versioning, migrations, repair, and cleanup commands for local ctxpack state.
- **STORE-04**: Preserve local-first, source-free defaults through storage, migration, and repair paths.

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

v1 and v1.1 are complete and archived. They validated:

- CLI, MCP, and public JSON compatibility guardrails.
- Safe inventory, diagnostics, privacy/source-read policy, and source-free local traces.
- Typed attributed retrieval candidates, context plans, context packs, historical evals, signal ablations, and gap reports.
- Codex CLI and Claude Code MCP smoke proof with explicit repo arguments.
- v1.1 binary packaging, checksums, artifact audit, install docs, setup validation, first-pack smoke, and release gates.

## Out of Scope

Explicitly excluded from v1.2 to prevent scope creep.

| Feature | Reason |
|---------|--------|
| New autonomous editing behavior | ctxpack remains a read-only context broker; agents own edits and permissions. |
| Production storage migration | v1.2 should prove which data deserves durable storage before building v1.3. |
| Embeddings or vector retrieval | v1.2 should measure existing retrieval gaps first; semantic retrieval belongs in v1.4. |
| SCIP/LSP precision integration | Parser precision belongs in v1.5 and should follow measured gap evidence. |
| Hosted team backend or source sync | Team/workspace capabilities belong in v2 and must preserve local-first trust. |
| GUI or pack inspector | UI belongs in v2.1 after benchmark/report artifacts exist. |
| Cloud embeddings, cloud reranking, telemetry, or remote source upload | Local-first privacy remains the product contract. |
| Claiming universal agent improvement | v1.2 must report limitations and failure cases honestly. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| BENCH-01 | Phase 9 | Pending |
| BENCH-02 | Phase 9 | Pending |
| BENCH-03 | Phase 9 | Pending |
| BENCH-04 | Phase 9 | Pending |
| METR-01 | Phase 10 | Pending |
| METR-02 | Phase 10 | Pending |
| METR-03 | Phase 10 | Pending |
| METR-04 | Phase 10 | Pending |
| METR-05 | Phase 10 | Pending |
| ROI-01 | Phase 10 | Pending |
| ROI-02 | Phase 10 | Pending |
| GAP-01 | Phase 11 | Pending |
| GAP-02 | Phase 11 | Pending |
| GAP-03 | Phase 11 | Pending |
| GAP-04 | Phase 11 | Pending |
| REG-01 | Phase 11 | Pending |
| REG-02 | Phase 11 | Pending |
| PROOF-01 | Phase 12 | Pending |
| PROOF-02 | Phase 12 | Pending |
| PROOF-03 | Phase 12 | Pending |
| PROOF-04 | Phase 12 | Pending |
| PROOF-05 | Phase 12 | Pending |

**Coverage:**
- v1.2 requirements: 22 total
- Mapped to phases: 22
- Unmapped: 0

---
*Requirements defined: 2026-05-14*
*Last updated: 2026-05-14 after v1.2 milestone initialization*
