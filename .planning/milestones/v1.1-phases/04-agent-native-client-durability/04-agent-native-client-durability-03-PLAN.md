---
phase: 04-agent-native-client-durability
plan: 03
type: execute
wave: 2
depends_on: [01]
files_modified:
  - crates/ctxpack-mcp/src/resources.rs
  - crates/ctxpack-mcp/src/lib.rs
  - crates/ctxpack/tests/cli_compat.rs
autonomous: true
requirements: [AGNT-02, AGNT-03]
must_haves:
  truths:
    - "Users can tell from diagnostics and the pack guide that MCP pack resources are session-scoped."
    - "A pack resource URI returned by prepare_task works in the same server process and returns a clear diagnostic after a new server process starts."
    - "MCP pack cache growth is bounded or explicitly test-visible without adding a new MCP tool."
    - "Wrong-working-directory behavior remains covered by explicit-repo tool tests from Plan 01."
  artifacts:
    - path: "crates/ctxpack-mcp/src/resources.rs"
      provides: "Session-scoped pack resource diagnostics and bounded/test-visible cache behavior"
    - path: "crates/ctxpack-mcp/src/lib.rs"
      provides: "In-process MCP resource/cache tests"
    - path: "crates/ctxpack/tests/cli_compat.rs"
      provides: "Subprocess restart regression for session-scoped pack resources"
  key_links:
    - from: "crates/ctxpack-mcp/src/tools.rs"
      to: "crates/ctxpack-mcp/src/resources.rs"
      via: "prepare_task cache_pack_resources"
      pattern: "cache_pack_resources"
    - from: "crates/ctxpack/tests/cli_compat.rs"
      to: "ctxpack serve-mcp"
      via: "two separate stdio server invocations"
      pattern: "resources/read"
---

<objective>
Make MCP pack resource session semantics and cache growth visible and tested.

Purpose: Users should not be surprised when a pack URI is tied to one MCP server process, and long-running clients should not grow an invisible unbounded cache.
Output: Clear session-scoped diagnostics, subprocess restart coverage, and bounded/test-visible pack cache behavior.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/phases/04-agent-native-client-durability/04-CONTEXT.md
@.planning/phases/04-agent-native-client-durability/04-RESEARCH.md
@.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-01-SUMMARY.md
@crates/ctxpack-mcp/src/resources.rs
@crates/ctxpack-mcp/src/lib.rs
@crates/ctxpack/tests/cli_compat.rs

<decision_trace>
- D-03: Pack resource session/restart semantics and cache growth must be visible and tested.
- D-05: Avoid new MCP tools unless the current surface cannot represent the durability contract.
</decision_trace>

<interfaces>
From `crates/ctxpack-mcp/src/resources.rs`:
```rust
pub(crate) fn read_resource(params: Value) -> Result<Value, RpcError>;
pub(crate) fn cache_pack_resources(
    repo: &Path,
    task: &str,
    plan: &ctxpack_core::ContextPlan,
    target_agent: &str,
) -> Result<(), RpcError>;
#[cfg(test)]
pub(crate) fn clear_pack_resource_cache();
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Characterize restart and session-scoped diagnostics</name>
  <files>crates/ctxpack-mcp/src/resources.rs, crates/ctxpack-mcp/src/lib.rs, crates/ctxpack/tests/cli_compat.rs</files>
  <behavior>
    - Test 1: `ctxpack://pack/guide` says pack resources are MCP-session scoped and `get_pack` is the durable reconnect-safe way to materialize a pack.
    - Test 2: Missing pack-resource errors include `session-scoped`, `same MCP server process`, and `call prepare_task first`.
    - Test 3: A subprocess test starts `ctxpack serve-mcp`, calls `prepare_task` with explicit repo and captures a pack URI, then starts a second server process and verifies `resources/read` for that URI returns the clear session-scoped diagnostic.
  </behavior>
  <action>Preserve session-scoped resources for AGNT-02 rather than adding reconstruction. Update `pack_guide_markdown` and `read_pack_resource` diagnostics to explain that pack URIs only work in the same MCP server process and that clients should call `get_pack` again after reconnect/restart. Add in-process tests in `crates/ctxpack-mcp/src/lib.rs` and a binary subprocess restart test in `crates/ctxpack/tests/cli_compat.rs`. The subprocess test should use the same fixture style as Plan 01 and must pass explicit `repo` to `prepare_task`; resource reads do not need a repo argument because the diagnostic is about pack cache session scope.</action>
  <verify>
    <automated>cargo test -p ctxpack-mcp pack_resource -- --nocapture && cargo test -p ctxpack --test cli_compat pack_resource -- --nocapture</automated>
  </verify>
  <done>Pack resources have explicit session-scoped diagnostics and a restart regression that proves the behavior across separate server processes.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Bound and test MCP pack cache growth</name>
  <files>crates/ctxpack-mcp/src/resources.rs, crates/ctxpack-mcp/src/lib.rs</files>
  <behavior>
    - Test 1: Repeated `prepare_task` calls do not leave more than the configured maximum number of cached pack resource entries.
    - Test 2: Eviction keeps the newest generated pack URI readable and evicts older entries with the same session-scoped diagnostic.
    - Test 3: Cache bound helpers are test-only or private; no public MCP tool or new external dependency is added.
  </behavior>
  <action>Add a small private cache limit in `resources.rs`, such as `MAX_PACK_RESOURCE_CACHE_ENTRIES`, and evict oldest keys deterministically after `cache_pack_resources` inserts Markdown and JSON variants. If BTreeMap ordering by URI is not insertion order, store a private insertion sequence alongside cached resources or a private VecDeque of keys so eviction is deterministic. Add `#[cfg(test)]` helpers only as needed to clear and inspect cache length. Keep the cache source-free and in-memory; do not persist packs or introduce a new MCP cache-inspection tool.</action>
  <verify>
    <automated>cargo test -p ctxpack-mcp pack_resource_cache -- --nocapture && cargo test --workspace</automated>
  </verify>
  <done>MCP pack cache growth is bounded and tested while the public tool/resource surface remains unchanged.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack-mcp pack_resource -- --nocapture`
- `cargo test -p ctxpack-mcp pack_resource_cache -- --nocapture`
- `cargo test -p ctxpack --test cli_compat pack_resource -- --nocapture`
- `cargo test --workspace`
- `cargo run -p ctxpack -- --help`
</verification>

<success_criteria>
Plan 03 is complete when pack resource URIs have clear same-session semantics, restart behavior is covered by a black-box subprocess test, and pack cache growth is bounded without adding a new MCP tool.
</success_criteria>

<output>
After completion, create `.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-03-SUMMARY.md`
</output>
