# Milestones

## Shipped

### v1.2 Retrieval Quality Proof (Shipped: 2026-05-14)

**Delivered:** Repeatable, source-free retrieval-quality proof with benchmark suites, fixed-budget metrics, baseline comparisons, gap taxonomy, trend comparison, and product proof generation.

**Phases completed:** Phases 9-12, 17 plans total

**Key accomplishments:**

- Added named benchmark suite contracts and bounded multi-repo historical evaluation with reproducibility and privacy metadata.
- Added fixed-budget file/test recall, lexical and no-context baselines, signal ablations, and token ROI reporting.
- Added source-free retrieval gap taxonomy, future-milestone recommendations, benchmark comparison, and regression thresholds.
- Added `ctxpack eval proof` plus optional `CTXPACK_BENCHMARK_CONFIG` release-gate proof.
- Kept benchmark, comparison, and proof artifacts source-free and local-only by default.

**Archive:**

- Roadmap: `.planning/milestones/v1.2-ROADMAP.md`
- Requirements: `.planning/milestones/v1.2-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.2-MILESTONE-AUDIT.md`

### v1.1 Packaging & Adoption (Shipped: 2026-05-13)

**Delivered:** A packaged, documented, smoke-testable ctxpack release path for agent-native adoption.

**Phases completed:** Phases 1-8, 32 plans total

**Key accomplishments:**

- Locked compatibility and source-free contract guardrails across CLI, MCP, and JSON outputs.
- Hardened safe inventory, diagnostics, context planning, packs, eval traces, and historical retrieval reports.
- Verified agent-native client durability through deterministic MCP proof and optional Codex/Claude real-client wrappers.
- Added v1.1.0 release identity, repeatable local binary archives, SHA-256 checksums, and artifact leakage audit.
- Added repo-local setup, `setup-check`, first-pack smoke, and thin guidance for Codex, Claude Code, Cursor, and OpenCode.
- Added docs and a release gate that verifies tests, docs, packaging, artifact audit, selected-binary behavior, MCP proof, and optional client wrappers.

**Archive:**

- Roadmap: `.planning/milestones/v1.1-ROADMAP.md`
- Requirements: `.planning/milestones/v1.1-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.1-MILESTONE-AUDIT.md`

## Active

### v1.3 Production Storage

**Goal:** Replace ad hoc JSON/cache behavior with durable, fast, local storage for inventory, symbols, history, traces, packs, benchmark results, and schema/version metadata.

**Why now:** v1.2 proved ctxpack's value with source-free benchmark evidence. The next adoption and scaling blocker is making repository intelligence, proof reports, and repeated benchmark comparisons durable and fast without weakening the local-first privacy contract.

**Planned phases:** Phases 13-16

**Definition of done:**

- Maintainer can initialize and inspect a versioned local SQLite store for repo metadata, symbols, chunks, edges, tests, git history, traces, packs, and benchmark results.
- Re-indexing a changed repo updates only stale records when possible and reports clear freshness, migration, and repair diagnostics.
- Benchmark, eval, pack, and proof metadata can be persisted and compared without storing source snippets or prompt text.
- Storage repair, cleanup, and release-gate checks preserve the source-free, read-only product contract.

## Planned Product Vision

### v1.4 Local Semantic Retrieval

**Goal:** Add optional local embedding/vector retrieval and hybrid fusion only after lexical/graph/history/test gaps justify it.

**Depends on:** v1.2 gap reports and v1.3 durable storage.

**Expected capabilities:**

- Local embedding provider interface with explicit privacy status.
- Vector candidate generation fused with lexical, graph, test, history, and active-context signals.
- Fixed-budget evals proving semantic lift over lexical/graph baselines.
- Cloud embeddings or reranking remain opt-in, visibly labeled, and disabled by default.

### v1.5 Parser/Semantic Precision

**Goal:** Improve symbol, dependency, and reference precision where measured failures show Tree-sitter/heuristics are insufficient.

**Depends on:** v1.2 gap taxonomy and v1.4 hybrid retrieval results.

**Expected capabilities:**

- Broader Tree-sitter language coverage where real repos need it.
- Optional SCIP/LSP import for precise definitions/references/call edges.
- Precision evals showing better target-file/test recall without context explosion.
- Safe degradation when language tooling is unavailable or project setup is incomplete.

### v2.0 Workspace & Team Layer

**Goal:** Support multi-repo and team workflows while keeping source code local and agent-native surfaces primary.

**Depends on:** v1.3 storage and stable source-free reporting.

**Expected capabilities:**

- Multi-repo workspace inventory and cross-repo context planning.
- Source-free shared context cards, policy files, and benchmark reports.
- Team-level privacy policy and adapter guidance templates.
- No hosted source indexing requirement.

### v2.1 UI / Pack Inspector

**Goal:** Add an optional diagnostics UI for inspecting packs, retrieval gaps, context health, and benchmark trends.

**Depends on:** v2.0 workspace metadata and v1.2/v1.3 reporting/storage foundations.

**Expected capabilities:**

- Pack inspector showing target files, evidence, token budgets, omitted candidates, and warnings.
- Retrieval-health dashboard for benchmark trends and repeated gap families.
- Context-card and adapter preview surfaces.
- UI remains diagnostic; daily coding still happens inside Codex, Claude Code, Cursor, OpenCode, and similar agents.
