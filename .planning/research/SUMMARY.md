# Project Research Summary

**Project:** Repo Context Packer
**Domain:** Local-first, agent-native repository context compiler for coding agents
**Researched:** 2026-05-13
**Confidence:** HIGH

## Executive Summary

Repo Context Packer is not a new coding agent, editor, or chat app. It is a local-first context compiler that helps existing agents inspect the right repository evidence for a task: target files, tests, graph neighbors, validation commands, safe snippets, current-diff anchors, and source-free provenance. Expert implementations in this domain keep the product surface agent-native and thin: small MCP tools/resources/prompts, portable repository instruction files, stable typed JSON contracts, and a CLI for setup/debug automation.

The recommended approach is to keep the current Rust workspace, typed contracts, safe inventory, budgeted packs, source-free traces, and small MCP surface, then harden the reliability layer before adding retrieval complexity. Fresh inventory, centralized privacy policy, safe source reads, structured diagnostics, and binary/client characterization tests should come before graph-ranking changes. Once trust is stable, graph/history/test/current-diff signals should feed an internal candidate-evidence model and be measured against fixed-budget lexical baselines.

The main risk is mistaking "more context" for better agent help. Stale cache, weak privacy filtering, silent partial failures, noisy graph edges, and shallow evals can all make ctxpack look useful while producing unsafe or irrelevant packs. Mitigate this with policy-gated retrieval, freshness checks on every user-facing read path, diagnostics-as-data, source-free historical eval with role-aware metrics, and phase gates that require measurable lift at fixed pack budgets.

## Key Findings

### Recommended Stack

Keep the existing Rust/Cargo workspace and layered crates. The stack already matches the product's local, read-only filesystem/git workload and has suitable boundaries for core contracts, indexing, compilation, MCP, and CLI. Add dependencies only when a phase has a measurable need: CLI test helpers and external-tool diagnostics now, parser-backed extraction later, and Tantivy/rayon/rusqlite/notify/rmcp only after the current contracts and eval gates are stable.

**Core technologies:**
- Rust + Cargo workspace: CLI, indexer, compiler, MCP server, and tests - keep the existing workspace and Rust 2021 baseline.
- serde/serde_json contracts: stable JSON for plans, packs, traces, eval reports, and MCP responses - preserve camelCase public contracts.
- clap derive CLI: setup/debug/automation surface - add binary-level tests before broad command expansion.
- ignore + blake3: safe inventory traversal and fast fingerprints - build freshness/privacy policy around them.
- MCP JSON-RPC stdio: agent-native integration - preserve the small stdio default and add protocol/client conformance tests.
- System git through a centralized runner: diff, history, co-change, and eval source of truth - add timeouts, stderr capture, version diagnostics, and batching.
- Tree-sitter parsers: optional parser-backed symbols/dependencies - add behind existing contracts for TS/TSX, Python, Rust, and Go after eval identifies the misses.

### Expected Features

The MVP baseline is already present: init artifacts, safe inventory, CLI/MCP workflows, hybrid retrieval, budgeted packs, current-diff anchors, source-free traces, historical eval, and deterministic context cards. Post-MVP work should focus on proving ctxpack beats agent grep, not expanding into a separate product surface.

**Must have (table stakes):**
- Agent-native setup artifacts - `AGENTS.md`, `.ctxpack/ctxpack.toml`, and thin native adapters.
- Small MCP tool/resource/prompt surface - keep `prepare_task`, `get_pack`, `search`, `related`, `related_tests`, and `current_diff` central.
- Safe repository inventory - respect ignore files, generated-file exclusions, and sensitive-path policy.
- Hybrid local retrieval - combine path anchors, lexical, symbol, current diff, dependency, tests, and co-change signals.
- Task-conditioned context plans - return files, tests, validation commands, pack options, risk flags, and diagnostics.
- Budgeted context packs - support brief, standard, and deep packs with safe snippets and source-free provenance.
- Operational diagnostics - expose stale cache, unreadable/skipped files, missing git, parse gaps, and low-signal tasks.
- Real client compatibility - verify Codex CLI, Claude Code, and adapter paths with explicit `repo` arguments.

**Should have (competitive):**
- Measured lift over lexical/grep baseline - historical eval becomes the product scorecard.
- Graph-first context expansion - dependency, co-change, symbol, test, and current-diff edges affect ranking.
- Evidence-set compiler - produce the smallest useful evidence set, not raw search results.
- Signal attribution - explain why each file/test/command was selected.
- Source-free provenance - keep traces/cards/evals usable without source or prompt leakage.
- Freshness-aware local cache - invalidate or warn before returning stale results.
- Persistent or reconstructable MCP pack resources - avoid session-fragile pack URIs for real clients.

**Defer (v2+):**
- Optional local semantic retrieval - only after lexical+symbol+graph evals show a remaining gap.
- Multi-repo/workspace context - after single-repo freshness, graph, and privacy are reliable.
- Team/shared index reuse and hosted enterprise sync - useful later, but outside current local-first hardening.
- Language-perfect static analysis - add parser-backed precision only where eval and fixtures justify it.

### Architecture Approach

Keep a layered Rust architecture: agent-facing CLI/MCP/adapter surfaces call stable core contracts and compiler facades, while retrieval internals evolve behind typed index/compiler modules. The critical architectural shift is making implicit subsystems explicit: privacy/freshness gates, safe source reads, diagnostics, candidate evidence, graph/history/test retrieval, and eval attribution.

**Major components:**
1. Core contracts - `ContextPlan`, `ContextPack`, `EvalTrace`, privacy status, repo discovery, and init artifacts.
2. Privacy and freshness gate - policy classification, safe inventory, cache metadata, ignore-file hashes, and snippet revalidation.
3. Index layer - lexical search, symbols, dependency/test/history/current-diff evidence, safe source reads, and diagnostics.
4. Compiler layer - candidate fusion, ranked plan projection, pack compilation, cards, eval, and risk-flag projection.
5. MCP boundary - thin JSON-RPC tool/resource/prompt handlers over compiler/index facades.
6. CLI boundary - human/debug automation rendering, with binary tests for wiring and output.

**Key patterns:**
- Policy-gated retrieval for every file-bearing API.
- Internal candidate graph with public plan projection to avoid breaking contracts.
- Diagnostics as structured data, not only free-text risk flags.
- Eval-driven retrieval changes with lexical baselines, graph lift, role recall, precision, and fixed budgets.

### Critical Pitfalls

1. **Treating cached inventory as ground truth** - store freshness metadata and require every read path to prove freshness, rebuild, or return stale-cache diagnostics.
2. **Privacy policy as a filename denylist** - centralize classification, add corpus tests for auth and credential patterns, and revalidate snippets immediately before reading.
3. **Context bloat masquerading as recall** - optimize recall and precision at fixed budgets, report pack size, and require selected files to carry meaningful evidence.
4. **Misleading dependency and symbol edges** - keep graph expansion deterministic, confidence-weighted, provenance-rich, and guarded by false-positive fixtures.
5. **Weak historical eval methodology** - freeze ranges, separate source/test/config/docs recall, add precision@k/MRR, handle rename/delete cases, and report signal ablations.
6. **Retrieval modalities without fusion discipline** - put score math in one module with candidate evidence, task-type weights, and golden ranking tests.
7. **MCP session semantics that only work in unit tests** - prefer durable `get_pack`, label session-scoped resources, or persist source-free plan metadata for regeneration.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Characterization, Boundaries, and CLI/MCP Contract Tests
**Rationale:** The codebase already has useful behavior; deeper refactors need protection around public JSON, Clap wiring, MCP shape, and existing retrieval results.
**Delivers:** Binary CLI tests, MCP protocol/handler tests, golden JSON shapes, and module extraction behind compatibility facades.
**Addresses:** Agent-native setup artifacts, small MCP surface, real client compatibility, typed contracts.
**Avoids:** Transport-driven retrieval logic, accidental public contract drift, and hidden command output regressions.

### Phase 2: Trust Layer - Freshness, Privacy, and Safe Reads
**Rationale:** Every retrieval, pack, resource, card, and eval label inherits correctness from safe inventory. Graph work is untrustworthy until stale cache and privacy gaps are closed.
**Delivers:** Freshness metadata, ignore/policy option hashes, stale-cache rebuild/diagnostics, centralized privacy policy, sensitive-path corpus tests, safe source reader, and snippet revalidation.
**Uses:** `ignore`, `blake3`, serde manifests, existing inventory contracts, optional `which` for tool diagnostics.
**Addresses:** Safe repository inventory, freshness-aware cache, configurable privacy policy, budgeted safe snippets.
**Avoids:** Stale inventory, privacy denylist sprawl, sensitive snippet leakage, read failures collapsed to empty strings.

### Phase 3: Diagnostics and Operational Visibility
**Rationale:** Weak plans must explain whether the issue is task ambiguity, stale cache, missing git, skipped files, parser gaps, unreadable content, or trace write failure.
**Delivers:** Shared `Diagnostic`/`OperationReport` types, risk-flag projection, CLI/MCP diagnostic fields, trace/cache write visibility, and weak-plan explanation tests.
**Addresses:** Operational diagnostics, weak-plan diagnostics, source-free trace controls.
**Avoids:** Opaque partial failures, surprising read-only write side effects, and user reindex rituals.

### Phase 4: Candidate Fusion and Measured Graph-Aware Planning
**Rationale:** ctxpack currently ties lexical baseline too often because graph/test/history evidence is not fully expressed as ranked candidates. This phase is the core product lift.
**Delivers:** `ContextCandidate`/`Evidence`/`SourceScores`, task-type-aware fusion, dependency/test/history/current-diff edge scoring, signal attribution, fixed-budget graph expansion, and golden ranking tests.
**Uses:** Existing lexical/symbol/test/dependency/co-change/current-diff signals; add parser adapters only if needed for fixture-backed edge quality.
**Addresses:** Graph-first ranking lift, signal attribution, related-test confidence, evidence-set compilation.
**Avoids:** Context bloat, misleading graph edges, uncalibrated heuristic piles, and graph boosts that only improve recall by increasing pack size.

### Phase 5: Evaluation Rigor and Retrieval Gap Reports
**Rationale:** Roadmap claims should be gated by source-free evals that show lift over lexical at fixed budgets, not by anecdotal pack quality.
**Delivers:** Frozen historical ranges, role-aware labels, rename/delete handling, precision@k/MRR, graph/history/test ablations, multiple real-repo smokes, and top gap reports by role/path family.
**Addresses:** Measured lift over lexical/grep baseline, historical retrieval gap reports, eval-driven roadmap decisions.
**Avoids:** Optimizing to noisy commit subjects, hiding lexical ties, and treating tests/config/docs as interchangeable with source targets.

### Phase 6: MCP and Client Operational Hardening
**Rationale:** Agent-native value depends on real client behavior, not just protocol unit tests. Pack resources and explicit repo handling need durable semantics.
**Delivers:** Reconnect/different-process resource tests, explicit `repo` guidance, pack resource scope labeling or source-free regeneration metadata, LRU session cache, non-fatal trace writes, and Codex/Claude/Cursor/OpenCode smoke scripts.
**Addresses:** Persistent/reconstructable MCP pack resources, real client compatibility, dynamic context discovery assets.
**Avoids:** Session-scoped resource surprises, cwd mismatch, process-local pack loss, and broad client support claims without client-path proof.

### Phase 7: Parser-Backed Precision and Optional Index Runtime Upgrades
**Rationale:** Parser and runtime dependencies should follow measured bottlenecks, not precede them. Once graph/eval gates exist, targeted parser work can improve edge quality without uncontrolled scope.
**Delivers:** Tree-sitter adapters for priority languages, resolver fixtures, edge confidence improvements, and measured decisions on Tantivy/rayon/rusqlite/notify/rmcp.
**Uses:** Tree-sitter for TS/TSX/Python/Rust/Go first; Tantivy/rayon/rusqlite/notify/rmcp only behind abstractions with before/after metrics.
**Addresses:** Parser-backed language plugins, large-repo performance, MCP protocol maturity.
**Avoids:** Parser migration before measurement, generic parser bundle bloat, premature vector/semantic search, and unnecessary daemon architecture.

### Phase Ordering Rationale

- Stabilize public contracts before refactors because CLI/MCP are already validated product surfaces.
- Build trust before retrieval lift because stale or unsafe inputs invalidate every ranking experiment.
- Add diagnostics before tuning because otherwise weak plans cannot distinguish product failure from weak tasks.
- Build a candidate/evidence model before graph expansion so new signals are explainable and measurable.
- Harden eval before claiming competitive differentiation because "beats grep" is the central product proof.
- Do client hardening after core semantics are clear, but before declaring broad agent support.
- Add parser/index/runtime upgrades last and only when phase metrics show where they matter.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4:** Fusion scoring, graph thresholds, task-type weights, and edge confidence require careful design and eval-backed tuning.
- **Phase 5:** Historical eval methodology needs benchmark design, frozen ranges, role labels, precision metrics, and multiple repo smokes.
- **Phase 7:** Tree-sitter resolver behavior, Tantivy index design, and rmcp migration should be researched only if selected by measured bottlenecks.

Phases with standard patterns (skip research-phase):
- **Phase 1:** CLI characterization, MCP shape tests, and module extraction are well-understood engineering work.
- **Phase 2:** Freshness manifests, privacy policy modules, safe readers, and corpus tests have clear local patterns in the codebase.
- **Phase 3:** Structured diagnostics and risk-flag projection are straightforward once the affected operations are enumerated.
- **Phase 6:** Client smoke tests and resource lifecycle checks are implementation-heavy but conceptually clear from existing MCP/client behavior.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Based on current workspace inventory, official crate docs, and stable local-first product constraints. Optional dependencies remain MEDIUM until measured in this codebase. |
| Features | HIGH | Current MVP state is documented in project/codebase files; competitor emphasis is MEDIUM because vendor docs do not expose ranking internals. |
| Architecture | HIGH | Current crate boundaries and proposed module splits are grounded in the codebase map and active requirements. External protocol/ecosystem direction is MEDIUM. |
| Pitfalls | HIGH | Risks are concrete in the current codebase and align across project concerns, MCP guidance, RAG evaluation framing, and historical eval observations. |

**Overall confidence:** HIGH

### Gaps to Address

- Fusion scoring design: define candidate evidence, score calibration, task-type weights, and graph admission thresholds during Phase 4 planning.
- Eval representativeness: add frozen ranges and at least two additional external real-repo smokes before making strong product claims.
- Privacy false positives: design repo-local allow/deny policy carefully so safety remains conservative without blocking legitimate source files.
- MCP resource durability: decide whether pack URIs are explicitly session-scoped or regenerated from persisted source-free metadata.
- Parser scope: choose languages and resolver features from observed misses, not from broad language coverage ambitions.
- Large-repo performance: measure repeated file reads and git-history fan-out before adding Tantivy, rayon, SQLite, or watch mode.

## Sources

### Primary (HIGH confidence)
- `.planning/PROJECT.md` - product goal, validated/active requirements, constraints, and key decisions.
- `.planning/research/STACK.md` - recommended technologies, versions, rationale, alternatives, and stack sequencing.
- `.planning/research/FEATURES.md` - table stakes, differentiators, anti-features, dependencies, and current MVP status.
- `.planning/research/ARCHITECTURE.md` - layered architecture, component responsibilities, patterns, data flow, and build order.
- `.planning/research/PITFALLS.md` - critical pitfalls, prevention strategies, phase mapping, and verification checks.
- `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md`, `.planning/codebase/CONCERNS.md`, `.planning/codebase/TESTING.md` - current codebase evidence cited by the research files.
- `README.md` and `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md` - current behavior and product direction cited by the research files.
- Model Context Protocol specification and docs - tools/resources/prompts, JSON-RPC architecture, security, and client best practices.
- Official crate docs for `ignore`, `tree-sitter`, `rmcp`, `tantivy`, `notify`, `rusqlite`, `assert_cmd`, and `trycmd`.
- Git documentation for `git log` changed-file and rename/delete metadata.

### Secondary (MEDIUM confidence)
- Cursor dynamic context discovery and secure codebase indexing posts - relevant vendor direction for context budgeting and indexing expectations.
- Sourcegraph Cody, Aider repo map, Claude Code memory, GitHub Copilot instructions, and VS Code Copilot instruction docs - ecosystem patterns for agent-native instructions and retrieval.
- SCIP overview and Tree-sitter project documentation - parser/code-intelligence integration direction.
- Retrieval-Augmented Code Generation survey, RANGER, CodexGraph, CodeRAG, CoIR, and FreshStack - research framing for repository-level retrieval, graph-enhanced retrieval, and evaluation.
- Ragas and LlamaIndex evaluation docs - retrieval precision/diagnostic metric framing.

### Tertiary (LOW confidence)
- Codebase-Memory arXiv preprint - useful signal for Tree-sitter knowledge graphs via MCP, but should not drive roadmap choices without local measurement.

---
*Research completed: 2026-05-13*
*Ready for roadmap: yes*
