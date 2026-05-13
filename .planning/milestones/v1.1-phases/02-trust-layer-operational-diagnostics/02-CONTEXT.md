# Phase 2: Trust Layer & Operational Diagnostics - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Mode:** Autonomous smart discuss; implementation choices at agent discretion within roadmap constraints.

<domain>
## Phase Boundary

This phase makes ctxpack's user-facing read paths trustworthy before retrieval-quality work continues. The delivered behavior should ensure search, planning, symbols, related tests, dependencies, packs, cards, and MCP resources either use fresh safe inventory data or return stable diagnostics explaining why results are stale, partial, skipped, or degraded.

The phase covers operational trust, not ranking improvements. Do not change retrieval scoring, add cloud/vector infrastructure, or broaden autonomous behavior. Keep ctxpack local-first and read-only in the product sense: local cache/trace/card writes are allowed only when explicit, controlled, and non-fatal for context retrieval.

</domain>

<decisions>
## Implementation Decisions

### Freshness Policy
- Prefer a centralized inventory freshness check over scattered per-call timestamp checks.
- User-facing read paths should rebuild stale inventory when safe and cheap enough, but return structured stale-cache diagnostics when rebuild fails or is disabled.
- Freshness should account for repository files, ignore files, inventory options, and deleted/renamed/sensitive/generated path changes.
- Preserve existing command behavior where possible; add diagnostics as compatible fields rather than replacing current outputs.

### Privacy And Source Reads
- Move sensitive/generated/binary/oversized/unreadable classification toward a centralized tested policy used by inventory, packs, file resources, cards, current diff, and historical labels.
- Revalidate every source-bearing path immediately before reading snippets for packs, MCP file resources, and generated cards.
- Treat package-manager auth files, SSH private keys, cloud credential JSON, credential-like dotfiles, vendored/generated output, binaries, non-UTF-8 files, oversized files, and unreadable files conservatively by default.
- Do not introduce cloud uploads, cloud embeddings, or remote reranking.

### Diagnostics Contract
- Add stable structured diagnostics for stale inventory, weak/low-information plans, missing git, git timeouts, unreadable files, skipped files, parse gaps, partial graph/test/history coverage, and cache/trace write failures.
- Preserve existing `riskFlags` compatibility while making diagnostics richer and more machine-readable for CLI and MCP clients.
- Diagnostics should be source-free: include paths, roles, reason codes, counts, hashes, and command/error categories, but not source snippets or prompt text.
- Prefer typed contracts in `ctxpack-core` over ad hoc strings in CLI/MCP renderers.

### Cache And Trace Writes
- Context retrieval should remain usable in constrained home-directory environments.
- Trace/cache write failures should be visible and non-fatal for read-oriented operations unless the user explicitly requested a write operation.
- Expose enough cache/trace status for users and tests to understand where local state is written and whether it was skipped.

### the agent's Discretion
- The agent may choose the exact Rust type names, module placement, and command flags if they preserve public compatibility and stay within the requirements.
- The agent may stage the work across multiple plans to protect the Phase 1 guardrails.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 1 added binary CLI guardrails in `crates/ctxpack/tests/cli_compat.rs` and shared temp-repo helpers in `crates/ctxpack/tests/common/mod.rs`.
- Public JSON compatibility tests now live in `crates/ctxpack-core/src/contracts.rs` and `crates/ctxpack-compiler/src/lib.rs`.
- MCP public-surface/session/error tests now live in `crates/ctxpack-mcp/src/lib.rs`.
- `ctxpack-index` is split into `inventory.rs`, `search.rs`, `symbols.rs`, `related_tests.rs`, `dependencies.rs`, `git.rs`, and `traces.rs`.
- `ctxpack-compiler` is split into `planning.rs`, `packs.rs`, `cards.rs`, and `eval.rs`.
- `ctxpack-mcp` is split into `protocol.rs`, `schemas.rs`, `tools.rs`, `resources.rs`, and `prompts.rs`.

### Established Patterns
- Use typed Rust contracts and camelCase serde output for public API fields.
- Use table-driven fixture tests for privacy, inventory, MCP, and CLI behavior.
- Use real temp git repositories for inventory, git, current-diff, historical, and CLI integration behavior.
- Use `serde_json::Value` shape assertions instead of full snapshots for compatibility surfaces.
- Keep public crate roots as stable facades while implementation moves behind focused modules.

### Integration Points
- Inventory freshness and privacy policy primarily attach to `crates/ctxpack-index/src/inventory.rs`.
- Source-bearing pack/card reads attach to `crates/ctxpack-compiler/src/packs.rs` and `crates/ctxpack-compiler/src/cards.rs`.
- MCP file/resource/tool diagnostics attach to `crates/ctxpack-mcp/src/resources.rs` and `crates/ctxpack-mcp/src/tools.rs`.
- CLI diagnostics attach to `crates/ctxpack/src/main.rs` while lower layers should expose typed data.
- Trace/cache write behavior attaches to `crates/ctxpack-index/src/traces.rs` and inventory persistence helpers.

</code_context>

<specifics>
## Specific Ideas

- Requirements in scope: SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, DIAG-01, DIAG-02, DIAG-04.
- Use deterministic fixtures for stale inventory after file create/delete/rename, ignore-file changes, sensitive/generated moves, unreadable files, non-UTF-8 data, oversized files, missing git, and read-only `CTXPACK_HOME`.
- Keep validation anchored by `cargo test --workspace` and `cargo run -p ctxpack -- --help`.
- Do not let this phase become the retrieval-lift phase; graph/ranking/eval improvements belong to Phase 3.

</specifics>

<deferred>
## Deferred Ideas

- DIAG-03 historical eval failure grouping belongs to Phase 3.
- Ranking lift, signal ablations, parser upgrades, and RefactoringMiner fixed-range eval gates belong to Phase 3.
- Real Codex/Claude client durability and MCP pack-resource persistence semantics belong to Phase 4.

</deferred>
