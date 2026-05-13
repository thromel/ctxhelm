# Requirements: Repo Context Packer

**Defined:** 2026-05-13
**Core Value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

## v1 Requirements

Requirements for the next post-MVP hardening and differentiation cycle. Each maps to roadmap phases.

### Contracts And Compatibility

- [ ] **CONT-01**: Maintainer can run binary-level CLI tests that exercise core commands and verify output shape, repo path handling, and write side effects.
- [ ] **CONT-02**: Maintainer can change internal modules without changing public JSON contracts for context plans, packs, eval traces, MCP structured content, and CLI outputs.
- [ ] **CONT-03**: Maintainer can run MCP handler/resource/prompt tests that verify current tool names, resource URI shapes, session behavior, and error responses.
- [ ] **CONT-04**: Maintainer can split large modules into focused submodules while preserving existing CLI, MCP, and library behavior.

### Trust And Safety

- [ ] **SAFE-01**: User-facing read paths detect stale inventory metadata before returning search, symbol, test, dependency, pack, card, or MCP results.
- [ ] **SAFE-02**: User-facing read paths either rebuild stale inventory or return structured stale-cache diagnostics that explain what changed.
- [ ] **SAFE-03**: ctxpack classifies sensitive and generated files through a centralized privacy policy with table-driven tests for common credential, auth, generated, vendored, and binary path families.
- [ ] **SAFE-04**: Pack, file-resource, and card generation revalidate every source-bearing path against the current safe inventory immediately before reading source text.
- [ ] **SAFE-05**: Unreadable, non-UTF-8, oversized, skipped, or externally unavailable inputs produce structured diagnostics instead of silently becoming empty matches.
- [ ] **SAFE-06**: Trace/cache writes are visible and controllable enough that context retrieval can remain usable in read-only or constrained home-directory environments.

### Diagnostics

- [ ] **DIAG-01**: User can see structured diagnostics on context plans for low-information tasks, stale cache, missing git, git timeout, unreadable files, skipped files, parse gaps, and partial graph/test/history coverage.
- [ ] **DIAG-02**: CLI and MCP outputs expose diagnostics in stable structured fields while preserving existing risk-flag compatibility.
- [ ] **DIAG-03**: Historical eval and checklist outputs summarize why retrieval failed, grouped by path role, signal gap, and repeated missing-file families.
- [ ] **DIAG-04**: Maintainer can test weak-plan scenarios with deterministic fixtures instead of relying on manual interpretation.

### Retrieval Quality

- [ ] **RETR-01**: Context planning represents candidate files, tests, symbols, docs, commits, and config as typed candidates with evidence and per-signal scores before projecting to `ContextPlan`.
- [ ] **RETR-02**: Dependency edges, related tests, co-change hints, current-diff anchors, and symbol matches can affect ranked target files, not only risk flags.
- [ ] **RETR-03**: Each recommended target file and related test includes source-free signal attribution explaining why it was selected.
- [ ] **RETR-04**: Graph expansion is budgeted and non-recursive by default so retrieval lift does not come from context bloat.
- [ ] **RETR-05**: Retrieval changes are evaluated against lexical baseline at fixed budgets and must show or explain lift on at least one frozen historical range.

### Evaluation

- [ ] **EVAL-01**: Maintainer can run frozen historical eval ranges with reproducible base/head refs, limits, mode, and source-free reports.
- [ ] **EVAL-02**: Historical eval handles additions, modifications, deletes, renames, generated files, sensitive files, source files, tests, configs, docs, and files that existed only in historical revisions.
- [ ] **EVAL-03**: Historical eval reports Recall@K, Precision@K, MRR or equivalent ranking quality, role-aware recall, test recommendation rate, lexical baseline, and signal ablations.
- [ ] **EVAL-04**: Maintainer can run large-repo smoke evals, including RefactoringMiner, without pathological full-worktree checkout costs.
- [ ] **EVAL-05**: Eval reports stay source-free and prompt-free while still providing enough gap detail to drive roadmap decisions.

### Agent-Native Operations

- [ ] **AGNT-01**: Codex CLI and Claude Code smoke scripts verify real MCP `prepare_task` and `get_pack` client paths with explicit `repo` arguments.
- [ ] **AGNT-02**: MCP pack resources are clearly session-scoped or can be reconstructed from persisted source-free metadata across server restarts.
- [ ] **AGNT-03**: MCP cache growth, reconnect behavior, and wrong-working-directory behavior are covered by tests or smoke scripts.
- [ ] **AGNT-04**: Generated adapter guidance stays thin and points agents to dynamic ctxpack calls rather than injecting large static context.

### Parser And Runtime Precision

- [ ] **PARS-01**: Maintainer can add parser-backed symbol and dependency adapters behind existing contracts without changing CLI or MCP output shapes.
- [ ] **PARS-02**: Parser-backed improvements are introduced only for languages and constructs with observed retrieval gaps or resolver false positives.
- [ ] **PARS-03**: Optional indexing/runtime upgrades such as Tantivy, rayon, SQLite, notify, or MCP SDK migration are evaluated with before/after metrics before adoption.

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Semantic Retrieval

- **SEMR-01**: User can enable optional local semantic retrieval when lexical, symbol, graph, and history signals leave measured gaps.
- **SEMR-02**: User can opt into cloud embeddings or reranking per repo with explicit privacy status and visibility into what leaves the machine.

### Workspace And Teams

- **WORK-01**: User can prepare context across multiple related repositories in one workspace.
- **WORK-02**: Team can share approved context cards, policies, and eval ranges without sharing private source snippets.
- **WORK-03**: Enterprise deployment can enforce policy, audit context outputs, and manage team-wide defaults.

### User Interface

- **UI-01**: User can inspect context packs, retrieval gaps, and diagnostics in an optional UI without making the UI the daily coding surface.
- **UI-02**: User can compare pack versions and signal contributions visually for debugging retrieval quality.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Autonomous source edits by ctxpack | Existing coding agents own editing, permissions, approvals, and shell execution. |
| Running user project test commands automatically | ctxpack should recommend verification commands; the agent or user should run them under their permission model. |
| Cloud indexing by default | Local-first privacy is part of the product contract. |
| Standalone chat/editor product | The product center is agent-native context delivery, not another daily app. |
| Large static repo context injection | It consumes tokens, gets stale, and undermines dynamic context discovery. |
| Broad parser migration before eval gates | Parser work should follow measured retrieval gaps, not precede them. |
| Hosted enterprise backend in current cycle | Reliability and measured local value are higher priority. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONT-01 | Phase 1 | Pending |
| CONT-02 | Phase 1 | Pending |
| CONT-03 | Phase 1 | Pending |
| CONT-04 | Phase 1 | Pending |
| SAFE-01 | Phase 2 | Pending |
| SAFE-02 | Phase 2 | Pending |
| SAFE-03 | Phase 2 | Pending |
| SAFE-04 | Phase 2 | Pending |
| SAFE-05 | Phase 2 | Pending |
| SAFE-06 | Phase 2 | Pending |
| DIAG-01 | Phase 2 | Pending |
| DIAG-02 | Phase 2 | Pending |
| DIAG-03 | Phase 3 | Pending |
| DIAG-04 | Phase 2 | Pending |
| RETR-01 | Phase 3 | Pending |
| RETR-02 | Phase 3 | Pending |
| RETR-03 | Phase 3 | Pending |
| RETR-04 | Phase 3 | Pending |
| RETR-05 | Phase 3 | Pending |
| EVAL-01 | Phase 3 | Pending |
| EVAL-02 | Phase 3 | Pending |
| EVAL-03 | Phase 3 | Pending |
| EVAL-04 | Phase 3 | Pending |
| EVAL-05 | Phase 3 | Pending |
| AGNT-01 | Phase 4 | Pending |
| AGNT-02 | Phase 4 | Pending |
| AGNT-03 | Phase 4 | Pending |
| AGNT-04 | Phase 4 | Pending |
| PARS-01 | Phase 3 | Pending |
| PARS-02 | Phase 3 | Pending |
| PARS-03 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 31 total
- Mapped to phases: 31
- Unmapped: 0

---
*Requirements defined: 2026-05-13*
*Last updated: 2026-05-13 after roadmap creation*
