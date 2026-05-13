# Phase 1: Compatibility Guardrails & Module Boundaries - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 1 delivers compatibility guardrails and safe module-boundary work for the existing Rust workspace. It protects current CLI behavior, MCP behavior, public JSON contracts, and library behavior before internal modules are split or refactored. It does not implement freshness, privacy-policy expansion, retrieval-ranking lift, historical-eval metric expansion, or real-client smoke scripts; those belong to later phases.

</domain>

<decisions>
## Implementation Decisions

### Compatibility strategy
- **D-01:** Characterization tests come before module splitting. Downstream planning should first lock current behavior through binary CLI tests, MCP tests, and contract-shape tests, then perform module extraction behind existing public functions.
- **D-02:** Preserve current public JSON field names, enum encodings, tool names, resource URI shapes, prompt names, and Markdown/report headings unless a test intentionally documents an additive backward-compatible change.
- **D-03:** Treat `ContextPlan`, `ContextPack`, `EvalTrace`, historical eval reports, MCP `structuredContent`, and major CLI JSON outputs as public compatibility surfaces for this phase.

### CLI guardrails
- **D-04:** Add binary-level CLI tests for representative core commands rather than only renderer/helper tests. The minimum command set should include `--help`, `index`, `prepare-task`, `get-pack`, `search`, `related-tests`, `dependencies`, `eval history`, and `serve-mcp` startup/protocol smoke where practical.
- **D-05:** CLI tests should use real temporary repositories and temporary `CTXPACK_HOME`, matching the existing fixture style in `.planning/codebase/TESTING.md`.
- **D-06:** Exact test harness choice is flexible, but the default planning assumption should be `assert_cmd`-style binary tests or an equivalent Cargo-supported approach that exercises the compiled binary rather than only direct function calls.

### MCP guardrails
- **D-07:** MCP compatibility tests must preserve the deliberately small implemented tool surface: `prepare_task`, `search`, `related`, `get_pack`, `related_tests`, and `current_diff`.
- **D-08:** MCP tests should cover both human-readable `content[0].text` and machine-readable `structuredContent` for changed tools, because real clients may consume either.
- **D-09:** Resource compatibility should cover repository resources, file/symbol resources, and session-scoped pack resources. It is acceptable in Phase 1 to characterize current session-scoped behavior rather than fix it; durability changes belong to Phase 4.

### Module boundaries
- **D-10:** Module splitting should follow existing crate responsibilities: keep public facades in each crate stable, move behavior into private modules, and avoid changing call sites outside the owning crate unless tests require a public facade update.
- **D-11:** Preferred first splits are by current concern, not by abstract utility: inventory/privacy-ish classification, lexical scoring/search, symbols, dependency graph, related tests, git/history/current diff, traces, planning, pack rendering, cards/eval, MCP schemas, MCP tools, MCP resources, and MCP prompts.
- **D-12:** Do not combine module splitting with new retrieval behavior. If a refactor reveals a bug, add a failing characterization/regression test first and keep the behavioral fix narrowly scoped.

### the agent's Discretion
- The planner may decide exact crate/module names, test crate layout, fixture helper names, and whether to use `assert_cmd`, `trycmd`, or direct `std::process::Command` binary tests, provided the result exercises real binary behavior and preserves existing public surfaces.
- The planner may decide how many golden fixtures are necessary, but must avoid brittle source-text snapshots where structured field assertions are more stable.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` — Defines Phase 1 goal, success criteria, dependency order, and covered requirements.
- `.planning/REQUIREMENTS.md` — Defines `CONT-01` through `CONT-04`, plus later-phase boundaries that Phase 1 must not absorb.
- `.planning/PROJECT.md` — Defines product principles: agent-native, local-first, read-only, typed contracts, and evaluation discipline.
- `.planning/STATE.md` — Current project state and pending concerns.

### Existing codebase map
- `.planning/codebase/ARCHITECTURE.md` — Current layered architecture and public entry points.
- `.planning/codebase/STRUCTURE.md` — Directory layout, where to add code, and crate ownership boundaries.
- `.planning/codebase/CONVENTIONS.md` — Naming, error handling, module style, and testing conventions to preserve.
- `.planning/codebase/TESTING.md` — Existing test patterns and fixture style.
- `.planning/codebase/CONCERNS.md` — Known risks that should remain later-phase work unless characterization tests are needed.

### Current public surfaces
- `Cargo.toml` — Workspace members and dependency policy.
- `crates/ctxpack/src/main.rs` — CLI command surface and report rendering.
- `crates/ctxpack-core/src/contracts.rs` — Public serializable contracts.
- `crates/ctxpack-index/src/lib.rs` — Current index/retrieval APIs that may be split behind stable facades.
- `crates/ctxpack-compiler/src/lib.rs` — Current planning, pack, cards, and eval APIs that may be split behind stable facades.
- `crates/ctxpack-mcp/src/lib.rs` — MCP JSON-RPC tools, resources, prompts, and session cache behavior.
- `README.md` — User-facing documented behavior that tests should not accidentally invalidate.
- `AGENTS.md` — Project working rules and required validation commands.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Inline fixture helpers in `crates/ctxpack-mcp/src/lib.rs`, `crates/ctxpack-index/src/lib.rs`, and `crates/ctxpack-compiler/src/lib.rs` already create temp repos, set `CTXPACK_HOME`, and run real git commands. New CLI binary tests should reuse the same pattern conceptually.
- Existing MCP tests in `crates/ctxpack-mcp/src/lib.rs` already exercise newline-delimited JSON-RPC requests through handler functions and should become compatibility references for any MCP module split.
- Existing contract tests in `crates/ctxpack-core/src/contracts.rs` already assert camelCase and enum serialization. Phase 1 should extend this style rather than introduce broad schema generation first.

### Established Patterns
- Library crates return typed errors and structs; CLI converts to `anyhow::Result`, while MCP converts to JSON-RPC errors. Module splits should preserve these boundaries.
- Tests are co-located in `#[cfg(test)] mod tests`, use behavior-oriented snake_case names, and prefer real filesystem/git fixtures over mocks.
- `crates/ctxpack-core` is already split into focused modules; `ctxpack-index`, `ctxpack-compiler`, and `ctxpack-mcp` are the large-module pressure points.

### Integration Points
- CLI compatibility work connects at `crates/ctxpack/src/main.rs`.
- Public JSON compatibility work connects at `crates/ctxpack-core/src/contracts.rs` and the report structs in `crates/ctxpack-compiler/src/lib.rs`.
- MCP compatibility work connects at `crates/ctxpack-mcp/src/lib.rs`, especially tool lists, resource lists, prompt lists, handler dispatch, and response serialization.
- Module-boundary work connects inside each owning crate and should preserve the crate root's public API where possible.

</code_context>

<specifics>
## Specific Ideas

- Treat Phase 1 as a protective harness, not a product-behavior phase.
- Prefer stable structured assertions over brittle full-output snapshots.
- Keep future retrieval-lift work out of Phase 1 unless a compatibility test is needed to protect current behavior before Phase 3 changes it.
- Keep current MCP pack-resource session semantics as characterized behavior; do not solve durability in Phase 1.

</specifics>

<deferred>
## Deferred Ideas

- Inventory freshness, safe source revalidation, privacy-policy expansion, and structured operational diagnostics belong to Phase 2.
- Candidate fusion, graph-ranked targets, signal attribution, parser-backed precision, and historical-eval metric expansion belong to Phase 3.
- Codex/Claude smoke scripts, MCP reconnect behavior, and pack-resource durability belong to Phase 4.

</deferred>

---

*Phase: 01-compatibility-guardrails-module-boundaries*
*Context gathered: 2026-05-13*
