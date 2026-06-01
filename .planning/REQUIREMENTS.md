# Requirements: ctxhelm v2.5 Production Retrieval Quality

**Defined:** 2026-05-22
**Milestone:** v2.5 Production Retrieval Quality
**Core Value:** Given a coding task, ctxhelm should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

## v2.5 Requirements

v2.4 made semantic, precision, provider, and reranker machinery policy-gated and source-safe. The fresh RefactoringMiner proof fixed the semantic regression, and v2.5 turned retrieval quality into measured lift across the fixed two-repo proof by evaluating context recall separately from validation-test recall.

### Multi-Repo Quality Baselines

- [x] **BASE-01**: Maintainer can run paired fixed-corpus quality baselines on at least RefactoringMiner and one second real repository using the same source-free report contract.
- [x] **BASE-02**: Reports preserve stable corpus identity, revision range, command options, provider status, runtime, cache status, and privacy status.
- [x] **BASE-03**: Reports identify named wins, misses, regressions, and repeated gap families without logging source text.
- [x] **BASE-04**: Baseline results can be compared across default, lexical, graph, semantic, reranked, and learned-policy variants with deterministic output.

### Production Local Embedding Quality

- [x] **EMBED-01**: `local_fastembed` or equivalent production local embedding backend can be built, indexed, queried, and evaluated without cloud transfer.
- [x] **EMBED-02**: Embedding cache behavior is bounded, source-safe, ignored by inventory, and reported with model/version/dimension metadata.
- [x] **EMBED-03**: Semantic retrieval quality is measured against lexical and default baselines before any default promotion.
- [x] **EMBED-04**: `local_hash` remains scaffold-labeled and cannot be mistaken for a quality backend in reports, docs, or MCP output.

### Reranker And Fusion Promotion

- [x] **RANK-01**: Local reranker/fusion variants can score first-stage candidates without adding MCP tool surface or exposing source text in reports.
- [x] **RANK-02**: Fusion policies protect explicit anchors, current diff, exact lexical evidence, and high-confidence symbols from semantic/graph crowd-out.
- [x] **RANK-03**: Promotion gates compare Recall@K, MRR@K, precision proxy, test recall, token ROI, and runtime before enabling any stronger default.
- [x] **RANK-04**: Regression cases are named and block promotion when they touch critical paths or exceed configured thresholds.

### Gap-Family Retrieval Improvements

- [x] **GAP-01**: Repeated `no_candidate_signal`, `lexical_only_miss`, `ranked_below_budget_*`, and test-mapping gaps are grouped into actionable work items.
- [x] **GAP-02**: At least one high-impact gap family from RefactoringMiner is improved with a targeted retrieval change and before/after proof.
- [x] **GAP-03**: Test recommendation quality is evaluated separately from source-file recall and does not silently trade away target files under tight budgets.
- [x] **GAP-04**: Graph expansion remains budgeted and does not recurse from weak semantic-only seeds when exact seeds exist.

### Product Proof And Release Gate

- [x] **PROOF-01**: v2.5 product proof states honestly whether production retrieval variants beat, match, or trail lexical baseline on each corpus.
- [x] **PROOF-02**: Release gate blocks default promotion for neutral, mixed, unsafe, or high-runtime variants.
- [x] **PROOF-03**: Docs explain which retrieval modes users should choose today and why.
- [x] **PROOF-04**: Workspace validation includes unit tests, CLI help, source-free E2E proof, and diff hygiene.

## Future Requirements

Deferred into later milestones from the remaining product vision.

### v2.6 Agent-Native Deep Integrations

- [x] **AGENT-01**: User can verify Codex and Claude Code real-client tool-call evidence from release docs.
- **AGENT-02**: User can verify Cursor and OpenCode integration behavior where clients expose machine-checkable proof.
- **AGENT-03**: User can install thin prompts/hooks/rules without broad static context injection.
- **AGENT-04**: User can use disconnected/cloud fallback cards when local MCP is unavailable.

### v2.7 Desktop Inspector & Local UX

- **UX-01**: User can open an optional desktop/local inspector shell for diagnostic review.
- **UX-02**: User can visualize graph neighborhoods and retrieval health interactively.
- **UX-03**: User can run setup/status checks from the UX without editing source files.
- **UX-04**: User can keep daily coding inside existing agents; the UX remains diagnostic.

### v3.0 Context Governor

- **GOVERN-01**: ctxhelm can adapt retrieval/budget/memory/validation policy per task.
- **GOVERN-02**: ctxhelm can learn from source-free agent sessions and eval outcomes.
- **GOVERN-03**: Maintainer can roll out, compare, and roll back context policies across repos.
- **GOVERN-04**: Maintainer can inspect why a context policy selected or omitted evidence.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Cloud embeddings or cloud reranking by default | Local-first trust remains the product contract. |
| Hosted vector database | v2.5 must prove local quality before adding hosted infrastructure. |
| Default semantic promotion without lift | Promotion still requires measured lift under source-free local policy gates. |
| Autonomous edits or test execution | Existing coding agents own editing, shell permissions, and validation execution. |
| Desktop inspector UX | v2.7 owns optional local UX. |
| Deep native hooks | v2.6 owns agent-native deep integration proof. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| BASE-01 | Phase 61 | Complete |
| BASE-02 | Phase 61 | Complete |
| BASE-03 | Phase 61 | Complete |
| BASE-04 | Phase 61 | Complete |
| EMBED-01 | Phase 62 | Complete |
| EMBED-02 | Phase 62 | Complete |
| EMBED-03 | Phase 62 | Complete |
| EMBED-04 | Phase 62 | Complete |
| RANK-01 | Phase 63 | Complete |
| RANK-02 | Phase 63 | Complete |
| RANK-03 | Phase 63 | Complete |
| RANK-04 | Phase 63 | Complete |
| GAP-01 | Phase 64 | Complete |
| GAP-02 | Phase 64 | Complete |
| GAP-03 | Phase 64 | Complete |
| GAP-04 | Phase 64 | Complete |
| PROOF-01 | Phase 65 | Complete |
| PROOF-02 | Phase 65 | Complete |
| PROOF-03 | Phase 65 | Complete |
| PROOF-04 | Phase 65 | Complete |
| AGENT-01 | Phase 70 | Complete |
| GAP-01 | Phase 71 | Complete |
| GAP-02 | Phase 71 | Complete |
| RANK-02 | Phase 71 | Complete |

**Coverage:**

- v2.5 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0
- Future integration requirement AGENT-01 has current Phase 70 evidence; remaining
  v2.6 integration requirements stay open.
- Phase 71 adds follow-up evidence for existing gap/ranking requirements by
  reducing archive-artifact noise without excluding archived evidence.

---
*Requirements defined: 2026-05-22*
