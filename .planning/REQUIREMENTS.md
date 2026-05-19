# Requirements: Repo Context Packer v2.3 Evaluation Lab & Learned Retrieval Policy

**Defined:** 2026-05-19
**Milestone:** v2.3 Evaluation Lab & Learned Retrieval Policy
**Core Value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

## v2.3 Requirements

This milestone turns ctxpack's retrieval-quality claims into repeatable, source-free, large-history product proof and introduces an offline learned-policy experiment that remains opt-in until it beats current baselines.

### Benchmark Corpora

- [x] **CORPUS-01**: Maintainer can define fixed benchmark corpus manifests with stable repo paths, revisions, commit ranges, budgets, task types, and privacy labels.
- [x] **CORPUS-02**: Maintainer can run a locked RefactoringMiner 20-commit regression suite that records the current Recall@10, lexical baseline, runtime, and known gap families as baseline metadata.
- [x] **CORPUS-03**: Maintainer can add additional local or public repositories to the same manifest format without storing source text, prompts, snippets, or private issue descriptions in reports.
- [x] **CORPUS-04**: Benchmark reports include reproducibility metadata: repo identity, revision range, index/compiler/policy versions, budget, task type, target agent, and source-free privacy status.

### Eval Speed And Reuse

- [x] **SPEED-01**: Historical eval can reuse warm parent snapshot, inventory, index, and candidate metadata when repo state, options, and versions have not changed.
- [x] **SPEED-02**: Historical eval can run commit samples in parallel and merge results deterministically with stable ordering and source-free output.
- [x] **SPEED-03**: Eval reports include runtime diagnostics for total time, per-commit time, cache hits, slow commits, git diff cost, ranking cost, and pack/compiler cost.
- [x] **SPEED-04**: Maintainer can compare stored runs and avoid recomputing unchanged benchmark ranges unless explicitly forced.

### Source-Free Candidate Features

- [x] **FEATURE-01**: Eval output can export source-free candidate feature rows for every considered file, symbol, test, doc, commit, memory, and graph candidate.
- [x] **FEATURE-02**: Candidate features include signal scores, rank positions, role/type metadata, graph distance, co-change/history metadata, test relation confidence, memory/feedback counts, and selected/read/edit/gold labels where available.
- [x] **FEATURE-03**: Feature exports are privacy-checked and do not include source snippets, prompt text, issue descriptions, terminal logs, stack traces, or secret-bearing values.
- [x] **FEATURE-04**: CLI and storage commands can list, inspect, compare, and delete feature exports by suite, run id, repo id, and privacy status.

### Paired Baselines And Ablations

- [x] **BASELINE-01**: Maintainer can compare ctxpack default ranking against lexical, no-context, graph-only, semantic-only, history-only, test-only, memory-only, and feedback-weighted policy variants on the same fixed corpus.
- [x] **BASELINE-02**: Reports include Recall@K, precision proxy, test recall, token ROI, validation coverage, missed-family taxonomy, signal saturation, runtime, and privacy status.
- [x] **BASELINE-03**: Product proof distinguishes lift, neutral, regression, and insufficient-evidence verdicts using configured thresholds instead of raw metric deltas alone.
- [x] **BASELINE-04**: Reports explicitly flag lexical parity or lexical regression so exact-token search strength is not hidden behind aggregate ctxpack scores.

### Learned Retrieval Policy

- [x] **POLICY-01**: Offline learner can propose source-free retrieval-policy weights from feature exports, historical eval labels, and feedback/outcome traces.
- [x] **POLICY-02**: Learned policy proposals are stored as non-default profiles with provenance, training corpus id, feature schema version, metric summary, and rollback metadata.
- [x] **POLICY-03**: Maintainer can compare, apply, disable, and roll back learned profiles through existing local policy controls without changing global defaults silently.
- [x] **POLICY-04**: Learned profiles cannot become default unless they beat configured baseline thresholds for recall, precision proxy, token ROI, runtime, validation coverage, and source-free privacy.

### Product Proof And Release Gates

- [ ] **PROOF-01**: Product proof can include a bounded v2.3 eval section covering fixed corpus identity, paired baseline verdicts, runtime diagnostics, feature-export privacy, and learned-policy status.
- [ ] **PROOF-02**: Release gate can run a small deterministic v2.3 eval smoke without requiring RefactoringMiner or other large external repos by default.
- [ ] **PROOF-03**: RefactoringMiner and multi-repo proof remain optional external gates with clear skip reasons, reproducible commands, and source-free artifacts.
- [ ] **PROOF-04**: Docs explain the proof boundary honestly: ctxpack may be useful at lexical parity, but world-class claims require repeated lift under fixed corpora and process-level context metrics.

## Future Requirements

Deferred into later milestones from the remaining product vision.

### v2.4 Production Semantic & Precision Backends

- **BACKEND-01**: Maintainer can use a production local vector index and real embedding backend with source-free metadata controls.
- **BACKEND-02**: Maintainer can enable cloud embeddings or reranking only through explicit repo policy gates.
- **BACKEND-03**: Maintainer can automate SCIP/LSP indexing for supported languages and report degraded precision inputs.
- **BACKEND-04**: Maintainer can migrate, compare, and roll back provider versions/dimensions using v2.3 eval gates.

### v2.5 Agent-Native Deep Integrations

- **AGENT-01**: User can verify Codex and Claude Code real-client tool-call evidence from release docs.
- **AGENT-02**: User can verify Cursor and OpenCode integration behavior where clients expose machine-checkable proof.
- **AGENT-03**: User can install thin prompts/hooks/rules without broad static context injection.
- **AGENT-04**: User can use disconnected/cloud fallback cards when local MCP is unavailable.

### v2.6 Desktop Inspector & Local UX

- **UX-01**: User can open an optional desktop/local inspector shell for diagnostic review.
- **UX-02**: User can visualize graph neighborhoods and retrieval health interactively.
- **UX-03**: User can run setup/status checks from the UX without editing source files.
- **UX-04**: User can keep daily coding inside existing agents; the UX remains diagnostic.

### v2.7 Team Sync & Enterprise Controls

- **TEAM-01**: Team can optionally sync source-free shared artifacts and policy metadata.
- **TEAM-02**: Admin can configure enterprise privacy/audit controls and SSO.
- **TEAM-03**: Team can expose a remote MCP endpoint only after explicit data-sharing review.
- **TEAM-04**: User can keep a fully local-only fallback when sync is disabled.

### v3.0 Context Governor

- **GOVERN-01**: ctxpack can adapt retrieval/budget/memory/validation policy per task.
- **GOVERN-02**: ctxpack can learn from source-free agent sessions and eval outcomes.
- **GOVERN-03**: Maintainer can roll out, compare, and roll back context policies across repos.
- **GOVERN-04**: Maintainer can inspect why a context policy selected or omitted evidence.

## Out of Scope

Explicitly excluded from v2.3 to keep the milestone focused on measured quality and learned policy foundations.

| Feature | Reason |
|---------|--------|
| Cloud embeddings or cloud reranking by default | Planned for v2.4 and must be justified by v2.3 eval gates. |
| Hosted benchmark service | Local-first trust and source-free proof remain the product boundary. |
| Full SWE-bench Pro execution harness | v2.3 can learn from SWE-bench Pro methodology without taking on full benchmark orchestration. |
| Production vector backend migration | Planned for v2.4 after source-free measurement proves where semantic retrieval helps. |
| SCIP/LSP automatic indexer installation | Planned for v2.4 after learned/eval gates can measure precision lift. |
| Real-client agent outcome execution as a required release blocker | Useful evidence, but environment-dependent and better suited to v2.5 deep integrations. |
| Learned policy as silent default | Learned profiles must stay opt-in until thresholds prove they improve over current and lexical baselines. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORPUS-01 | Phase 50 | Complete |
| CORPUS-02 | Phase 50 | Complete |
| CORPUS-03 | Phase 50 | Complete |
| CORPUS-04 | Phase 50 | Complete |
| SPEED-01 | Phase 51 | Complete |
| SPEED-02 | Phase 51 | Complete |
| SPEED-03 | Phase 51 | Complete |
| SPEED-04 | Phase 51 | Complete |
| FEATURE-01 | Phase 52 | Complete |
| FEATURE-02 | Phase 52 | Complete |
| FEATURE-03 | Phase 52 | Complete |
| FEATURE-04 | Phase 52 | Complete |
| BASELINE-01 | Phase 53 | Complete |
| BASELINE-02 | Phase 53 | Complete |
| BASELINE-03 | Phase 53 | Complete |
| BASELINE-04 | Phase 53 | Complete |
| POLICY-01 | Phase 54 | Complete |
| POLICY-02 | Phase 54 | Complete |
| POLICY-03 | Phase 54 | Complete |
| POLICY-04 | Phase 54 | Complete |
| PROOF-01 | Phase 55 | Planned |
| PROOF-02 | Phase 55 | Planned |
| PROOF-03 | Phase 55 | Planned |
| PROOF-04 | Phase 55 | Planned |

**Coverage:**
- v2.3 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0

---
*Requirements defined: 2026-05-19*
