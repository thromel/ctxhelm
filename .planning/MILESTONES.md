# Milestones

## Active

### v2.5 Production Retrieval Quality

**Goal:** Prove and improve retrieval quality across real repositories so production local embeddings, reranking, graph/test/history fixes, and learned fusion can beat lexical baseline while staying local-first and source-safe.

**Status:** Active: 2026-05-22. Phases 61-62 complete.

**Phases planned:** Phases 61-65, 5 plans total

**Planned capabilities:**

- Multi-repo fixed-corpus baselines across RefactoringMiner and a second real repository.
- Production local embedding quality proof with bounded local cache and provider metadata.
- Reranker and fusion promotion gates that protect anchors, current diff, lexical evidence, and symbols.
- Gap-family retrieval fixes with before/after proof on real misses.
- v2.5 product proof and release gate that honestly reports beat/match/trail status per corpus and variant.

**Active artifacts:**

- Roadmap: `.planning/ROADMAP.md`
- Requirements: `.planning/REQUIREMENTS.md`
- Phases: `.planning/phases/61-multi-repo-quality-baselines/` through `.planning/phases/65-v25-product-proof-release-gate/`

**Current evidence:**

- Phase 61 E2E: `.planning/e2e/2026-05-22-v25-multirepo-baseline.md`
- Phase 62 E2E: `.planning/e2e/2026-05-30-phase62-production-local-embedding-quality.md`

## Shipped

### v2.4 Production Semantic & Precision Backends

**Goal:** Convert semantic and precision retrieval from local scaffolding into measured, policy-gated retrieval-quality improvements without breaking ctxpack's local-first and source-safe contract.

**Status:** Shipped locally: 2026-05-20.

**Phases completed:** Phases 56-60, 5 plans total

**Key accomplishments:**

- Added an optional production-local `local_fastembed` backend behind the `local-embeddings` feature, while keeping `local_hash` as deterministic scaffold behavior.
- Rebuilt semantic retrieval around source-free semantic documents enriched with safe inventory, symbols, dependency edges, related tests, docs/cards, and precision status.
- Added source-free query construction traces and hybrid fusion controls for task, commit, path, symbol, and error-like inputs.
- Added provider and reranker policy gates with cloud embeddings, source transfer, and cloud reranking disabled by default.
- Added semantic/precision evaluation gates and release proof boundaries so defaults are not promoted without measured lift.
- Verified Claude Code can pass semantic provider/model/dimension controls through MCP.
- Ran the fresh RefactoringMiner paired proof: default and `local_hash` are now at parity after the semantic seed fix, but both still trail lexical baseline.

**Artifacts:**

- Roadmap: `.planning/ROADMAP.md`
- Requirements: `.planning/REQUIREMENTS.md`
- Research: `.planning/research/`
- Audit: `.planning/milestones/v2.4-MILESTONE-AUDIT.md`
- Fresh E2E: `.planning/e2e/2026-05-22-refactoringminer-semantic-fusion-regression.md`
- Phases: `.planning/phases/56-production-local-semantic-backend/` through `.planning/phases/60-semantic-precision-evaluation-gates-and-release-proof/`

### v2.3 Evaluation Lab & Learned Retrieval Policy

**Goal:** Make ctxpack's retrieval-quality claims repeatable across fixed corpora, large histories, policy variants, and source-free learned retrieval experiments.

**Status:** Shipped locally: 2026-05-19.

**Phases completed:** Phases 50-55, 6 plans total

**Key accomplishments:**

- Added fixed source-free benchmark corpus manifests and locked RefactoringMiner v2.3 baseline metadata.
- Added cached and deterministic parallel historical eval with runtime diagnostics.
- Added source-free candidate feature exports for learning, diagnostics, and paired analysis.
- Added paired baseline and ablation analysis with lexical lift/parity/regression verdicts.
- Added offline learned retrieval-policy proposals with thresholded application and rollback controls.
- Added v2.3 product proof summary, deterministic eval smoke, and release-gate proof boundary docs.

**Archive:**

- Roadmap: `.planning/milestones/v2.3-ROADMAP.md`
- Requirements: `.planning/milestones/v2.3-REQUIREMENTS.md`
- Audit: `.planning/milestones/v2.3-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v2.3-phases/`

### v2.2 Release & Distribution Hardening

**Goal:** Turn the locally complete ctxpack product into a reproducible, installable, public release with clean packaging, upgrade, adoption, and proof artifacts.

**Status:** Shipped locally: 2026-05-19.

**Phases completed:** Phases 45-49, 5 plans total

**Key accomplishments:**

- Added clean-checkout release gate and source-free release proof bundle.
- Added install, upgrade, setup-check, troubleshooting, and agent setup docs.
- Added public adoption docs, static demo artifacts, and first-pack guidance.
- Added distribution metadata, clean extraction verification, and signing/notarization gap docs.
- Added release governance, candidate lifecycle, and rollback documentation.

**Archive:**

- Roadmap: `.planning/milestones/v2.2-ROADMAP.md`
- Requirements: `.planning/milestones/v2.2-REQUIREMENTS.md`
- Audit: `.planning/milestones/v2.2-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v2.2-phases/45-clean-release-gate-proof-bundle/` through `.planning/milestones/v2.2-phases/49-release-governance-candidate-lifecycle/`

### v2.1 Pack Inspector & GraphRAG Retrieval

**Goal:** Add an optional diagnostic pack inspector and measured GraphRAG/embedding retrieval improvements while keeping ctxpack local-first, read-only, and agent-native.

**Status:** Shipped locally: 2026-05-18.

**Phases completed:** Phases 39-44, 6 plans total

**Key accomplishments:**

- Added source-free pack inspector contracts plus JSON, Markdown, and static HTML exports.
- Added a local read-only inspector UI with filters, responsive layout checks, and sentinel leak tests.
- Added retrieval-health reports for historical eval trends, feedback gaps, signal contribution, and token ROI.
- Added source-free graph neighborhood/community reports and policy experiment comparisons.
- Added semantic provider status and explicit cloud-disabled embedding/reranking controls.
- Added agent previews for Codex CLI, Claude Code, Cursor, OpenCode, and generic MCP clients.
- Wired inspector, health, graph, policy/embedding, and agent-preview smokes into release docs and release-gate contracts.

**Archive:**

- Roadmap: `.planning/milestones/v2.1-ROADMAP.md`
- Requirements: `.planning/milestones/v2.1-REQUIREMENTS.md`
- Audit: `.planning/milestones/v2.1-MILESTONE-AUDIT.md`
- Research: `.planning/milestones/v2.1-research/`
- Phases: `.planning/milestones/v2.1-phases/`

### v2.0 Workspace & Team Layer (Shipped locally: 2026-05-17)

**Delivered:** Multi-repo workspace manifests/status, workspace-aware context plans and packs, source-free shared artifact manifests, local team privacy policy templates, MCP workspace resources, docs, and release smokes.

**Phases completed:** Phases 35-38, 4 plans total

**Key accomplishments:**

- Added source-free workspace manifests and status aggregation across multiple local repositories.
- Added workspace-aware `prepare-task` and `get-pack` with repo-boundary-preserving `repoPacks`.
- Added shared artifact export, inspect, and import flows plus local team privacy policy reports.
- Added MCP resources for `ctxpack://workspace/status` and `ctxpack://workspace/shared-artifacts` without expanding the six-tool MCP surface.
- Added workspace/shared-artifact docs and release-gate smoke coverage.

**Archive:**

- Roadmap: `.planning/milestones/v2.0-ROADMAP.md`
- Requirements: `.planning/milestones/v2.0-REQUIREMENTS.md`
- Research: `.planning/milestones/v2.0-research/`
- Phases: `.planning/milestones/v2.0-phases/`

### v1.7 Adaptive Retrieval Policy & Feedback Loop (Shipped: 2026-05-17)

**Delivered:** Source-free feedback events, policy quality reports, adaptive policy profile controls, agent outcome comparison, feedback docs, and release-gate smoke coverage.

**Phases completed:** Phases 30-34, 5 plans total

**Key accomplishments:**

- Added source-free session feedback contracts and local JSONL ingestion/list/summary CLI.
- Added policy quality reports for context precision, read precision, edit recall proxy, validation coverage, repeated missing-file families, signal contribution, and token ROI.
- Added explicit local retrieval-policy profile controls for tune, list, apply, disable, and rollback.
- Added outcome comparison by plan-only, brief, standard, and deep pack budgets with low-evidence warnings.
- Added `docs/feedback.md`, `scripts/smoke-feedback.sh`, release-doc checks, and release-gate coverage.

**Archive:**

- Roadmap: `.planning/milestones/v1.7-ROADMAP.md`
- Requirements: `.planning/milestones/v1.7-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.7-MILESTONE-AUDIT.md`
- Phases: `.planning/milestones/v1.7-phases/`

### v1.6 Repo Memory & Experience Cards (Shipped: 2026-05-16)

**Delivered:** Source-free repo memory cards, experience cards, review controls, and selected-memory plan/pack output.

**Phases completed:** Phases 25-29, 5 plans total

**Key accomplishments:**

- Added source-free memory card contracts and SQLite `memory_cards` persistence.
- Generated freshness-aware domain cards from safe inventory, symbols, tests, docs, and dependency edges.
- Generated source-free experience cards from local eval traces and structured metadata.
- Selected memory into `prepare_task`, `get_pack`, and MCP resources under explicit evidence and token-budget caps.
- Added memory review/redaction/disable/regeneration commands, docs, and deterministic smoke coverage.

**Archive:**

- Roadmap: `.planning/milestones/v1.6-ROADMAP.md`
- Requirements: `.planning/milestones/v1.6-REQUIREMENTS.md`
- Audit: `.planning/milestones/v1.6-MILESTONE-AUDIT.md`
- Research: `.planning/milestones/v1.6-research/`
- Phases: `.planning/milestones/v1.6-phases/`

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

### v2.5 Agent-Native Deep Integrations

**Goal:** Make ctxpack feel native inside Codex, Claude Code, Cursor, OpenCode, and generic MCP clients without taking over editing or shell execution.

**Depends on:** v1.1 setup/adoption, v2.1 agent previews, v2.2 release installation path.

**Expected capabilities:**

- Stronger Codex and Claude Code real-client proof with request-log artifacts.
- Cursor and OpenCode proof paths where the clients expose machine-checkable evidence.
- Agent-specific prompts/hooks/rules that stay thin, dynamic, and repo-local.
- Cloud/disconnected fallback cards for agents that cannot reach local MCP.

### v2.6 Desktop Inspector & Local UX

**Goal:** Package the diagnostic inspector as a polished optional local UX for understanding and debugging ctxpack decisions.

**Depends on:** v2.1 static inspector and agent preview, v2.2 release packaging.

**Expected capabilities:**

- Optional Tauri or native desktop shell around the local inspector.
- Graph visualization for source-free neighborhoods and communities.
- Onboarding/status checks for setup, storage, benchmark proof, and agent config.
- No daily coding UI: the desktop surface remains diagnostic and read-only.

### v2.7 Team Sync & Enterprise Controls

**Goal:** Add optional team-safe sharing and governance without weakening local-first defaults.

**Depends on:** v2.0 shared artifacts/team policy, v2.2 release trust, v2.4 provider policy controls.

**Expected capabilities:**

- Optional remote metadata sync for source-free cards, benchmark reports, policy profiles, and shared artifacts.
- Enterprise privacy/audit policy, SSO/admin controls, and explicit data-sharing review.
- Remote MCP endpoint for approved source-free or policy-allowed context.
- Clear local-only fallback with no hosted dependency.

### v3.0 Context Governor

**Goal:** Turn ctxpack from a context compiler into an adaptive context governor for AI coding agents.

**Depends on:** v2.3 learned policy, v2.4 semantic/precision backends, v2.5 integrations, v2.7 governance.

**Expected capabilities:**

- Adaptive per-task budget, retrieval, memory, graph, semantic, and validation policy.
- Closed-loop learning from source-free agent sessions and eval outcomes.
- Policy rollout, rollback, and comparison across repos/teams.
- Context-quality decisions exposed clearly enough for maintainers to trust and tune.
