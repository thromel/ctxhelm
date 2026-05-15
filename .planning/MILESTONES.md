# Milestones

## Active

No active milestone. Next planned milestone is `v1.6 Repo Memory & Experience Cards`.

## Shipped

### v1.5 Parser/Semantic Precision (Shipped: 2026-05-16)

**Delivered:** Java/Kotlin parser precision plus source-free precision edge import for local SCIP/LSP bridge outputs.

**Phases completed:** Phases 21-24, 4 plans total

**Key accomplishments:**

- Added Java/Kotlin symbol extraction for safe inventoried source/test files.
- Added Java/Kotlin dependency graph inference for safe local package imports and common source-root layouts.
- Added `ctxpack precision import` with source-free edge validation and `.ctxpack/precision-edges.json` persistence.
- Added additive `precision:<edgeType>` dependency output without changing existing graph contracts.
- Added `docs/precision.md`, `scripts/smoke-precision.sh`, release-doc checks, and release-gate coverage.
- Verified parser precision on the RefactoringMiner repository.

**Archive:**

- Roadmap: `.planning/milestones/v1.5-ROADMAP.md`
- Requirements: `.planning/milestones/v1.5-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.5-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v1.5-phases/`

### v1.4 Local Semantic Retrieval (Shipped: 2026-05-16)

**Delivered:** Optional local semantic retrieval as a measured, source-free, local-only signal inside the context compiler.

**Phases completed:** Phases 17-20, 16 plans total

**Key accomplishments:**

- Added typed semantic provider metadata with disabled-by-default invocation flags.
- Added schema v2 source-free semantic vector metadata with incremental reuse counts.
- Added local semantic search, `--semantic` CLI support, and additive MCP `semantic` arguments for existing workflows.
- Fused semantic candidates as a secondary retrieval signal behind exact path, active diff, symbol, lexical, graph, and test evidence.
- Added semantic-enabled historical eval metadata, `docs/semantic.md`, and deterministic semantic release-gate smoke coverage.

**Archive:**

- Roadmap: `.planning/milestones/v1.4-ROADMAP.md`
- Requirements: `.planning/milestones/v1.4-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.4-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v1.4-phases/`

### v1.3 Production Storage (Shipped: 2026-05-14)

**Delivered:** Durable, source-free SQLite storage for repository intelligence, incremental inventory sync, pack/eval/proof metadata persistence, storage operations, docs, and release-gate smoke coverage.

**Phases completed:** Phases 13-16, 16 plans total

**Key accomplishments:**

- Added a versioned source-free SQLite schema with metadata, migration history, and privacy labels.
- Added `ctxpack index --store` with reused/created/updated/deleted safe file record counts.
- Added source-free pack, historical eval, benchmark, retrieval-gap, and proof metadata persistence.
- Added `ctxpack storage init/status/repair/vacuum/reset` with reset dry-run behavior.
- Added `docs/storage.md` and `scripts/smoke-storage.sh`, wired into release docs and release gate.

**Archive:**

- Roadmap: `.planning/milestones/v1.3-ROADMAP.md`
- Requirements: `.planning/milestones/v1.3-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.3-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v1.3-phases/`
- Research: `.planning/milestones/v1.3-research/`

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

## Planned Product Vision

### v1.6 Repo Memory & Experience Cards

**Goal:** Turn generated repo summaries and prior agent-run lessons into durable, source-linked, source-free memory that agents can reuse without bloating every context pack.

**Depends on:** v1.3 durable storage, v1.4 semantic retrieval, and v1.5 precision signals.

**Expected capabilities:**

- Domain cards for key subsystems with freshness metadata, source links, and regeneration triggers.
- Experience cards from prior agent sessions, test failures, accepted fixes, and user corrections.
- Memory selection in `prepare_task` and `get_pack` with explicit evidence and token-budget caps.
- Review and redaction workflow that keeps generated memory source-free by default and editable by humans.

### v1.7 Adaptive Retrieval Policy & Feedback Loop

**Goal:** Use benchmark results and real agent-session traces to tune retrieval policy, identify repeated gaps, and prove whether context choices improve agent outcomes.

**Depends on:** v1.2 benchmark proof, v1.3 storage, v1.4 semantic retrieval, v1.5 precision, and v1.6 memory cards.

**Expected capabilities:**

- Session feedback ingestion for recommended, read, edited, tested, passed, failed, and user-corrected files.
- Source-free policy statistics for signal weights, context precision, token ROI, and repeated missing-file families.
- Eval-driven retrieval-policy tuning with rollback when semantic, graph, history, precision, or memory signals regress.
- Agent outcome reports comparing plan-only, brief, standard, and deep packs across fixed tasks.

### v2.0 Workspace & Team Layer

**Goal:** Support multi-repo and team workflows while keeping source code local and agent-native surfaces primary.

**Depends on:** v1.3 storage, v1.6 source-free memory, and v1.7 feedback reporting.

**Expected capabilities:**

- Multi-repo workspace inventory and cross-repo context planning.
- Source-free shared context cards, policy files, and benchmark reports.
- Team-level privacy policy templates without hosted source indexing.
- No hosted source indexing requirement.

### v2.1 UI / Pack Inspector

**Goal:** Add an optional diagnostics UI for inspecting packs, retrieval gaps, context health, and benchmark trends.

**Depends on:** v2.0 workspace metadata and v1.2/v1.3 reporting/storage foundations.

**Expected capabilities:**

- Pack inspector showing target files, evidence, token budgets, omitted candidates, and warnings.
- Retrieval-health dashboard for benchmark trends and repeated gap families.
- Context-card and adapter preview surfaces.
- UI remains diagnostic; daily coding still happens inside Codex, Claude Code, Cursor, OpenCode, and similar agents.
