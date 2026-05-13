# Repo Context Packer

## What This Is

Repo Context Packer is a local-first, read-only context broker that helps existing coding agents choose better repository context. It does not replace Codex, Claude Code, Cursor, OpenCode, Aider, or similar tools; it exposes task-conditioned file, test, graph, history, and pack guidance through agent-native surfaces such as MCP, AGENTS.md, and thin adapter files.

The current codebase is a Rust workspace with a CLI, MCP server, safe repository inventory, lexical and symbol retrieval, related-test inference, dependency hints, current-diff anchors, context packs, generated context cards, local eval traces, and historical retrieval evaluation.

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

### Active

- [ ] Make graph expansion first-class in the context plan, not only risk-flag evidence, so ctxpack can beat lexical baseline on historical evals.
- [ ] Add inventory freshness checks so search, planning, symbols, tests, dependency graph, and MCP results do not rely on stale cached file metadata.
- [ ] Strengthen privacy classification beyond the current denylist, especially for package-manager auth files, SSH keys, cloud credentials, and sensitive JSON names.
- [ ] Add operational diagnostics for weak or partial context plans, including stale cache, unreadable files, skipped large files, git timeout, and missing external-tool signals.
- [ ] Add binary-level CLI tests for the main commands so Clap wiring, output formats, repo path handling, and write side effects are covered outside library unit tests.
- [ ] Improve historical retrieval evaluation quality, including rename/delete cases and graph/history lift measurement on large real repositories such as RefactoringMiner.
- [ ] Split the largest modules along stable boundaries without changing public contracts: inventory/privacy, search/scoring, symbols, dependencies, tests, git/history, traces, planning, packs, cards, eval, MCP schemas, MCP tools, and MCP resources.

### Out of Scope

- Autonomous code editing inside ctxpack — existing coding agents already own editing, permissions, approvals, and shell execution.
- Cloud indexing, cloud embeddings, or cloud reranking by default — local-first trust is part of the product contract.
- A standalone daily chat app or editor replacement — ctxpack should improve agent-native workflows instead of becoming another coding surface.
- Hosted backend, team sync, SSO, or enterprise admin — useful later, but not part of the current product hardening.
- Broad language-perfect semantic analysis — parser-backed precision is valuable, but the current priority is measured context quality behind stable contracts.

## Context

The product started from the thesis that code agents do not need another generic repo chat app; they need a context compiler that decides which files, tests, examples, constraints, and snippets belong in the model context for a specific task. The existing implementation follows that shape: a Rust workspace separates contracts, indexing/retrieval, context compilation, MCP transport, and CLI rendering.

The codebase map in `.planning/codebase/` documents the current system:

- `crates/ctxpack-core/src/contracts.rs` defines the stable typed contracts consumed by CLI and MCP.
- `crates/ctxpack-index/src/lib.rs` owns safe inventory, search, symbols, test mapping, dependency edges, git history, current diff, historical samples, and eval traces.
- `crates/ctxpack-compiler/src/lib.rs` owns context-plan fusion, pack compilation, context-card generation, Markdown rendering, provenance, and historical eval.
- `crates/ctxpack-mcp/src/lib.rs` owns JSON-RPC/MCP tools, resources, prompts, session-scoped pack cache, and tool/resource response shaping.
- `crates/ctxpack/src/main.rs` owns the user-facing CLI and command output.

The current proof point is mixed. The product is useful enough to provide agent-native context, targeted tests, source-free local traces, and real MCP integration. However, on the RefactoringMiner historical eval slice, ctxpack currently ties lexical baseline at Recall@10, even after improving Recall@5 from `0.29` to `0.43`. The next product work must create measurable lift over lexical retrieval, not merely more features.

## Constraints

- **Privacy**: Default behavior must stay local-only and source-safe for inventory, plans, traces, historical eval reports, and generated cards. Packs may contain safe snippets, but every snippet path must remain filtered through the safe inventory policy.
- **Product surface**: AGENTS.md, MCP, and thin native rules/adapters remain the primary surfaces. CLI exists for setup, debugging, and automation, not as the daily product center.
- **Read-only scope**: ctxpack should not edit source code, run user project tests, install dependencies, or auto-commit user work. It can write its own local caches, traces, generated cards, adapter files, and planning/docs artifacts.
- **Implementation stack**: Keep the current Rust workspace architecture and typed contracts unless there is a clear measured reason to change.
- **Evaluation**: New retrieval work should be checked against source-free historical evals, with RefactoringMiner as a large-history external smoke target when practical.
- **Validation**: Run `cargo test --workspace` before claiming implementation work complete, and `cargo run -p ctxpack -- --help` after CLI changes.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Agent-native integration first | The product should improve Codex, Claude Code, Cursor, OpenCode, and similar tools rather than forcing a separate daily workflow. | ✓ Good |
| Local-first and read-only by default | Developer trust depends on not uploading source or taking over editing/approval responsibilities. | ✓ Good |
| Rust workspace core | The product needs fast local filesystem/git work, deterministic CLI behavior, and small distributable binaries. | ✓ Good |
| Small MCP tool surface | Too many MCP tools increase context overhead and client decision complexity. | ✓ Good |
| Source-free eval traces and historical reports | Retrieval quality should be measured without storing prompt text or source snippets. | ✓ Good |
| Use real repositories for proof | RefactoringMiner exposes scale, history, and retrieval-quality failures that synthetic fixtures miss. | — Pending |

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
*Last updated: 2026-05-13 after initialization*
