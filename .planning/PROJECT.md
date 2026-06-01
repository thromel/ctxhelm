# ContextMason

## What This Is

ContextMason, powered by the `ctxpack` CLI, is a local-first, read-only context broker that helps existing coding agents choose better repository context. It does not replace Codex, Claude Code, Cursor, OpenCode, Aider, or similar tools; it exposes task-conditioned file, test, graph, history, and pack guidance through agent-native surfaces such as MCP, AGENTS.md, and thin adapter files.

The current codebase is a Rust workspace with a CLI, MCP server, safe repository inventory, lexical, symbol, semantic, and precision-edge retrieval, related-test inference, dependency hints, current-diff anchors, context packs, generated context cards, local eval traces, historical retrieval evaluation, benchmark suites, source-free retrieval gap reporting, product proof generation, release hardening, and diagnostic inspector surfaces.

## Current State: v2.4 Production Semantic & Precision Backends Planned

The v2.3 milestone is complete locally and archived in the milestone index. ctxpack now has fixed source-free benchmark corpora, cached and deterministic parallel historical eval, candidate feature exports, paired baseline and ablation analysis, offline learned retrieval-policy proposals, and v2.3 product proof gates wired into release validation.

The May 19 semantic ablation showed no RefactoringMiner Recall@10 lift from the current `local_hash` semantic scaffold and worse runtime. v2.4 is planned to turn semantic and precision retrieval into production-quality, policy-gated infrastructure through real local embeddings, precision-enriched semantic documents, query construction, optional provider/reranker gates, and fixed-corpus promotion gates.

## Current Milestone

v2.4 Production Semantic & Precision Backends.

Goal: Convert semantic and precision retrieval from local scaffolding into measured, policy-gated retrieval-quality improvements without breaking ctxpack's local-first and source-safe contract.

Planned phases:

- Phase 56: Production Local Semantic Backend
- Phase 57: Precision-Enriched Semantic Documents
- Phase 58: Query Construction And Hybrid Fusion Controls
- Phase 59: Provider And Reranker Policy Gates
- Phase 60: Semantic/Precision Evaluation Gates And Release Proof

## Core Value

Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

## Requirements

### Validated

- ✓ Repository initialization generates portable `AGENTS.md`, `.ctxpack/ctxpack.toml`, and optional thin native adapter artifacts for Cursor, Claude Code, and OpenCode.
- ✓ The CLI exposes the main local workflows: `init`, `index`, `prepare-task`, `get-pack`, `search`, `symbols`, `related-tests`, `co-changes`, `dependencies`, `cards`, `eval`, and `serve-mcp`.
- ✓ The MCP server exposes a deliberately small tool surface: `prepare_task`, `search`, `related`, `get_pack`, `related_tests`, and `current_diff`.
- ✓ Safe inventory excludes ignored, generated, and sensitive paths by default and persists local inventory state under `CTXPACK_HOME` or `~/.ctxpack`.
- ✓ Retrieval combines lexical search, symbol search, related tests, local dependency edges, current-diff anchors, and local co-change hints into task-conditioned context plans.
- ✓ Context packs can be emitted as structured JSON or Markdown with budgeted sections, validation commands, snippets from safe files, and source-free provenance fields.
- ✓ Local eval traces and historical retrieval evals are source-free and report file recall, lexical baseline comparison, missing files, and privacy status.
- ✓ Generated context cards provide source-free repo summaries, testing summaries, and dependency graph summaries for cloud or disconnected agent contexts.
- ✓ End-to-end local MCP smoke has been verified for Claude Code and Codex CLI with explicit `repo` arguments.
- ✓ Phase 1 validated compatibility guardrails and module boundaries: binary CLI tests, public JSON contract tests, MCP protocol/resource/prompt tests, and focused index/compiler/MCP modules behind stable crate-root facades.
- ✓ Phase 2 validated the trust layer: stale inventory detection/rebuild diagnostics, centralized source-read policy, safe pack/card/file-resource revalidation, structured CLI/MCP diagnostics, and non-fatal trace/cache write handling.
- ✓ Phase 3 validated measured retrieval lift gates: typed retrieval candidates, source-free attribution, fixed-budget ranking, status-aware historical eval labels, lexical comparison, signal ablations, grouped retrieval gaps, checklist reporting, CLI/MCP compatibility, and bounded RefactoringMiner smoke.
- ✓ Phase 4 validated agent-native client durability: deterministic MCP protocol smoke from wrong cwd, required-mode Codex CLI and Claude Code real-client smokes with server-side `prepare_task`/`get_pack` evidence, explicit session-scoped pack resource diagnostics, bounded MCP pack cache behavior, and thin dynamic adapter guidance.
- ✓ v1.1 validated repeatable local release artifacts, SHA-256 checksums, source-build fallbacks, and release artifact leakage audit.
- ✓ v1.1 validated repo-local agent setup, actionable `ctxpack init` reporting, read-only `setup-check`, and first-pack smoke through deterministic MCP proof.
- ✓ v1.1 validated installed-binary quickstart docs, agent setup matrix, troubleshooting docs, and proof-boundary documentation for Codex, Claude Code, Cursor, and OpenCode.
- ✓ v1.1 validated a local release gate covering workspace tests, docs consistency, packaging/audit, selected-binary behavior, first-pack smoke, wrong-cwd MCP proof, and optional Codex/Claude wrappers.
- ✓ v1.2 validated named benchmark suites for RefactoringMiner and additional real repositories with reproducible, source-free revision and budget metadata.
- ✓ v1.2 validated fixed-budget file/test recall, lexical and no-context baseline deltas, signal ablations, and token ROI reporting.
- ✓ v1.2 validated source-free retrieval gap taxonomy, future milestone recommendations, benchmark comparison, and regression thresholds.
- ✓ v1.2 validated product proof generation, benchmark proof documentation, and optional release-gate benchmark smoke via `CTXPACK_BENCHMARK_CONFIG`.
- ✓ v1.3 validated durable source-free SQLite storage, incremental indexing metadata, pack/eval/proof persistence, storage operations, and storage release gates.
- ✓ v1.4 validated optional local semantic retrieval, source-free vector metadata, semantic fusion, semantic eval flags, documentation, and release-gate smoke coverage.
- ✓ v1.5 validated Java/Kotlin symbol and dependency precision plus source-free SCIP/LSP bridge edge import.
- ✓ v1.6 validated source-free memory cards, experience cards, review controls, selected-memory plan/pack output, docs, and memory smoke coverage.
- ✓ v1.7 validated source-free feedback events, policy quality reports, adaptive profile controls, outcome comparison, feedback docs, and release-gate smoke coverage.
- ✓ v2.0 validated multi-repo workspaces, workspace-aware plans and packs, source-free shared artifacts, team privacy policy templates, MCP workspace resources, docs, and release smokes.
- ✓ v2.1 validated source-free pack inspector exports, local static inspector UI, retrieval-health reports, graph neighborhoods, policy/embedding controls, agent previews, and release-gate smokes for those diagnostic surfaces.
- ✓ v2.3 validated fixed benchmark corpora, cached and parallel historical eval, source-free candidate feature exports, paired baselines, offline learned retrieval-policy proposals, product proof gates, and deterministic v2.3 release smoke.

### Active

- Build a production local semantic backend with a real embedding provider while keeping source text local and reports source-free.
- Enrich semantic retrieval with typed symbol/test/docs context and optional SCIP/LSP precision status.
- Add structured query construction and paired hybrid fusion controls for task, commit, explicit-path, symbol, and error-like inputs.
- Keep cloud embeddings and reranking disabled by default behind explicit repo policy gates.
- Block semantic, precision, or reranker defaults unless fixed-corpus gates prove measurable lift over existing baselines.

### Out of Scope

- Autonomous code editing inside ctxpack — existing coding agents already own editing, permissions, approvals, and shell execution.
- Cloud indexing, cloud embeddings, or cloud reranking by default — local-first trust is part of the product contract.
- A standalone daily chat app or editor replacement — ctxpack should improve agent-native workflows instead of becoming another coding surface.
- Hosted backend, team sync, SSO, or enterprise admin — useful later, but not part of v2.4 production semantic/precision backend work.
- Cloud indexing, cloud embeddings, and cloud reranking by default — v2.4 may add explicit provider policy gates, but defaults stay local-only.
- Full SWE-bench Pro execution harness — ctxpack should learn from contamination-aware benchmark design without taking on a hosted benchmark platform.
- Real-client agent outcome execution as a required release blocker — useful later, but environment-dependent and better suited to v2.5 deep integrations.

## Context

The product started from the thesis that code agents do not need another generic repo chat app; they need a context compiler that decides which files, tests, examples, constraints, and snippets belong in the model context for a specific task. The existing implementation follows that shape: a Rust workspace separates contracts, indexing/retrieval, context compilation, MCP transport, and CLI rendering.

The codebase map in `.planning/codebase/` documents the current system:

- `crates/ctxpack-core/src/contracts.rs` defines the stable typed contracts consumed by CLI and MCP.
- `crates/ctxpack-index/src/lib.rs` is the stable facade for safe inventory, freshness, source-read policy, search, symbols, test mapping, dependency edges, git history, current diff, status-aware historical samples, and eval traces implemented in focused index modules.
- `crates/ctxpack-compiler/src/lib.rs` is the stable facade for diagnostic-aware context-plan fusion, typed retrieval ranking, pack compilation, source revalidation, context-card generation, Markdown rendering, provenance, fixed-budget historical eval, benchmark suite execution, gap taxonomy, comparison, and proof reporting.
- `crates/ctxpack-mcp/src/lib.rs` is the stable facade for JSON-RPC/MCP protocol, tools, resources, prompts, diagnostics, session-scoped pack cache, and tool/resource response shaping implemented in focused MCP modules.
- `crates/ctxpack/src/main.rs` owns the user-facing CLI and command output.

v1 through v1.1 proved the local context broker, source-free safety model, agent-native protocol surface, packaging path, and setup/release gates. v1.2 proved the adoption claim with measured retrieval-quality evidence. v1.3 converted those measured needs into production-grade local storage. v1.4 used that storage foundation to add local semantic retrieval only as a measured, optional signal inside the existing context compiler. v1.5 added parser and precision-edge coverage where Java-heavy real repos exposed structural gaps. v1.6 turned durable repo summaries and session lessons into selective, source-free memory. v1.7 closed the learning loop by comparing ctxpack recommendations with what agents actually read, edit, test, and validate. v2.0 extended those local, source-free foundations across multi-repo workspaces and team-safe artifacts. v2.1 turned the accumulated evidence into inspectable diagnostic surfaces and added measured graph/embedding controls. v2.2 made the product installable, reproducible, and credible for public adoption. v2.3 turned the research and local evidence into repeatable evaluation, fast iteration, paired baselines, and source-free learned retrieval policy. v2.4 now uses that proof layer to build production semantic and precision backends only where fixed-corpus gates justify the added complexity.

Milestone strategy from the original product vision:

- **v1.4 Local Semantic Retrieval**: add optional local embeddings/vector retrieval with hybrid fusion and explicit privacy controls.
- **v1.5 Parser/Semantic Precision**: expand Tree-sitter coverage and add optional SCIP/LSP precision only where measured gaps justify it.
- **v1.6 Repo Memory & Experience Cards**: shipped; source-free memory cards and experience cards that agents can reuse selectively.
- **v1.7 Adaptive Retrieval Policy & Feedback Loop**: shipped; use benchmark results and real session traces to report/tune retrieval policy and compare agent outcomes.
- **v2.0 Workspace & Team Layer**: shipped locally; support multi-repo workspaces, shared source-free context cards, benchmark reports, and team policy files.
- **v2.1 Pack Inspector & GraphRAG Retrieval**: shipped locally; optional diagnostics UI, retrieval health, graph neighborhoods, policy/embedding controls, and agent previews.
- **v2.2 Release & Distribution Hardening**: shipped locally; clean release packaging, install/upgrade paths, public proof bundle, docs, and adoption artifacts.
- **v2.3 Evaluation Lab & Learned Retrieval Policy**: shipped locally; fixed benchmark corpora, fast historical evals, candidate feature exports, paired baselines, policy learning, regression thresholds, and honest product proof boundaries.
- **v2.4 Production Semantic & Precision Backends**: production local vector index, optional cloud providers, reranking gates, and deeper SCIP/LSP automation.
- **v2.5 Agent-Native Deep Integrations**: stronger Codex/Claude/Cursor/OpenCode hooks, prompts, previews, and real-client proof expansion.
- **v2.6 Desktop Inspector & Local UX**: optional desktop/Tauri inspector shell, graph visualization, onboarding, and diagnostics workflow polish.
- **v2.7 Team Sync & Enterprise Controls**: optional remote/team metadata sync, policy audit, admin controls, SSO, and remote MCP.
- **v3.0 Context Governor**: adaptive context-policy engine that controls budget, evidence, validation, memory, and agent feedback across tasks.

## Constraints

- **Privacy**: Default behavior must stay local-only and source-safe for inventory, plans, traces, historical eval reports, benchmark reports, generated cards, and product proof. Packs may contain safe snippets, but every snippet path must remain filtered through the safe inventory policy.
- **Product surface**: AGENTS.md, MCP, and thin native rules/adapters remain the primary surfaces. CLI exists for setup, debugging, and automation, not as the daily product center.
- **Read-only scope**: ctxpack should not edit source code, run user project tests, install dependencies, or auto-commit user work. It can write its own local caches, traces, generated cards, adapter files, and planning/docs artifacts.
- **Implementation stack**: Keep the current Rust workspace architecture and typed contracts unless there is a clear measured reason to change.
- **Evaluation**: New retrieval and storage work should be checked against source-free historical evals, with RefactoringMiner as a large-history external smoke target when practical.
- **Validation**: Run `cargo test --workspace` before claiming implementation work complete, and `cargo run -p ctxpack -- --help` after CLI changes.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Agent-native integration first | The product should improve Codex, Claude Code, Cursor, OpenCode, and similar tools rather than forcing a separate daily workflow. | ✓ Good |
| Local-first and read-only by default | Developer trust depends on not uploading source or taking over editing/approval responsibilities. | ✓ Good |
| Rust workspace core | The product needs fast local filesystem/git work, deterministic CLI behavior, and small distributable binaries. | ✓ Good |
| Small MCP tool surface | Too many MCP tools increase context overhead and client decision complexity. | ✓ Good |
| Source-free eval traces and historical reports | Retrieval quality should be measured without storing prompt text or source snippets. | ✓ Good |
| Use real repositories for proof | RefactoringMiner exposes scale, history, and retrieval-quality failures that synthetic fixtures miss. | ✓ Validated in Phase 3 smoke and v1.2 benchmark proof |
| Compatibility guardrails before refactors | Binary CLI, JSON contract, and MCP compatibility tests should fail before internal module changes drift public behavior. | ✓ Validated in Phase 1 |
| Trust diagnostics before retrieval lift | Measured retrieval work should build on fresh inventory, safe source reads, and explicit degraded-result diagnostics. | ✓ Validated in Phase 2 |
| Measured retrieval before client durability | Real clients should consume ranked, attributed, source-free context surfaces, not unstable heuristic output. | ✓ Validated in Phase 3 |
| Real-client proof must be machine-checkable | Agent-native durability requires server-side or transcript evidence for `prepare_task` and `get_pack`, not final assistant prose. | ✓ Validated in Phase 4 |
| Release artifacts must preserve clean-checkout semantics | Publication gates should not silently package dirty source unless maintainers explicitly opt into an in-flight validation override. | ✓ Validated in v1.1 audit |
| Deterministic smokes must not write into target repos | ctxpack's read-only product contract applies to release validation as well as runtime agent use. | ✓ Validated in v1.1 audit |
| Deterministic protocol proof is the required gate | Real Codex/Claude client proof is valuable but remains optional and environment-gated; deterministic MCP proof is the portable release blocker. | ✓ Validated in Phase 8 |
| Retrieval proof before bigger architecture | Storage, embeddings, parser precision, team features, and UI should be justified by measured retrieval gaps instead of speculative architecture desire. | ✓ Validated in v1.2 |
| Precision overlays should be source-free bridges first | Direct SCIP/LSP project setup is fragile; a source-free edge overlay gives agents precise structure without making language tooling mandatory. | ✓ Validated in v1.5 |
| Eval lab before heavier backends | Research and local RefactoringMiner evidence show the next bottleneck is repeatable proof and learned selection policy, not adding another retrieval backend blindly. | Adopted for v2.3 |
| Real semantic backends need hard gates | The May 19 semantic ablation showed the hash scaffold produced no Recall@10 lift and worse runtime, so v2.4 must treat stronger embeddings, precision, and reranking as gated hypotheses. | Adopted for v2.4 |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check - still the right priority?
3. Audit Out of Scope - reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-19 after v2.4 production semantic and precision backend planning*
