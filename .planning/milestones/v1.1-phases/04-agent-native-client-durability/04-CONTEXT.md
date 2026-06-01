# Phase 4: Agent-Native Client Durability - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase hardens the agent-native runtime path after Phase 3 made retrieval output measurable and attributed. Users should be able to rely on ctxhelm from real coding-agent clients without confusing session behavior, wrong-repo assumptions, or bulky static adapter context.

The phase covers real Codex CLI and Claude Code smoke scripts, explicit `repo` arguments for MCP tools, MCP pack resource semantics across server restarts, cache growth/reconnect/wrong-working-directory tests, and generated adapter guidance that tells agents to call dynamic ctxhelm tools instead of injecting large static repository summaries.

This phase does not add new retrieval signals, cloud/vector search, autonomous editing, or a standalone app surface.

</domain>

<decisions>
## Implementation Decisions

### Real Client Smokes
- Add scriptable smoke paths for Codex CLI and Claude Code when the client binary is installed, but keep deterministic protocol-level tests as the local fallback.
- Smokes must exercise actual MCP `prepare_task` and `get_pack` flows with explicit `repo` arguments, not only config discovery or server startup.
- Scripts should skip clearly when a client is unavailable and fail only when a requested/available smoke path violates the ctxhelm contract.

### MCP Repo And Pack Semantics
- Prefer explicit repo arguments in MCP tools over relying on the MCP server process working directory.
- Make pack-resource semantics visible to users and tests: either clearly session-scoped or reconstructable from persisted source-free metadata.
- Test restart/reconnect and wrong-working-directory behavior directly so clients do not silently read from the wrong repo or fail with unclear resource errors.
- Bound or expose MCP pack cache growth so long-running clients do not accumulate invisible unbounded state.

### Adapter Guidance
- Keep generated AGENTS.md, Cursor rules, Claude commands, OpenCode config/rules, and any Codex guidance thin.
- Adapter text should direct agents to call ctxhelm dynamically for non-trivial code tasks and to load packs progressively.
- Do not inject large generated repo context into adapter rules. Static cards are acceptable only as concise fallback context for environments where dynamic MCP is unavailable.

### Scope Control
- Do not add autonomous source editing to ctxhelm.
- Do not widen the MCP tool list unless a current tool cannot represent the durability contract.
- Do not add cloud indexing, cloud embeddings, cloud reranking, or vector search.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ctxhelm-mcp/src/` owns the JSON-RPC/MCP server, tool schemas, tool handlers, resources, prompts, and session-scoped pack cache.
- `crates/ctxhelm/src/main.rs` owns CLI commands, including `serve-mcp`, `init`, adapter generation, and smoke-friendly JSON output.
- `crates/ctxhelm/tests/cli_compat.rs` already verifies binary CLI surfaces and can host generated adapter and smoke-script compatibility checks.
- Phase 1 locked MCP public tool names and resource URI shapes.
- Phase 2 added MCP diagnostics, safe file resource revalidation, and controllable trace writes.
- Phase 3 added attributed `prepare_task` output, fixed-budget eval reporting, CLI/MCP compatibility tests, and `scripts/smoke-historical-eval.sh`.

### Established Patterns
- Keep MCP tools small and stable: `prepare_task`, `search`, `related`, `get_pack`, `related_tests`, and `current_diff`.
- Preserve public JSON compatibility with additive fields and `serde(default)`.
- Use source-free diagnostics and reports for degraded behavior.
- Add scriptable smokes under `scripts/` with clear environment variables and skip behavior.
- Run `cargo test --workspace` and `cargo run -p ctxhelm -- --help` before claiming completion.

### Integration Points
- MCP `prepare_task` and `get_pack` must accept explicit repo input and return structured diagnostics when repo resolution fails.
- MCP resources such as `ctxhelm://pack/{id}/brief` currently depend on process/session state and need explicit documented/testable behavior.
- Adapter generation should be checked where it writes `AGENTS.md`, `.cursor/rules`, `.claude/commands`, and OpenCode config/rules.
- Real client smokes should be optional but runnable on this machine when `codex` or Claude Code CLI commands exist.

</code_context>

<specifics>
## Specific Ideas

- Add `scripts/smoke-codex-mcp.sh` and `scripts/smoke-claude-mcp.sh`, or a shared smoke harness with client-specific wrappers.
- Add a deterministic MCP JSON-RPC smoke that launches `ctxhelm serve-mcp` via stdio and calls `prepare_task` and `get_pack` with explicit repo arguments; use it as fallback when real clients are absent.
- Add tests proving `get_pack` resource reads after a restart either reconstruct from source-free metadata or return a clear session-scoped diagnostic.
- Add tests for wrong working directory: server started outside repo, explicit `repo` points to target repo, and output comes from that repo.
- Add tests or diagnostics for MCP cache size/eviction/limits.
- Add adapter snapshot tests proving generated guidance remains thin and dynamic.

</specifics>

<deferred>
## Deferred Ideas

- A remote MCP endpoint for cloud agents is future work.
- Persisted team context cards, hosted sync, and enterprise policy are future milestones.
- Visual UI for inspecting pack cache/resource durability is out of scope.

</deferred>
