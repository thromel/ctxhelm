---
phase: 06-agent-setup-first-pack-adoption
plan: 02
type: execute
wave: 2
depends_on: [01]
files_modified:
  - crates/ctxpack-core/src/init.rs
autonomous: true
requirements: [ADPT-02, ADPT-03]
must_haves:
  truths:
    - "Generated Codex, Claude Code, Cursor, and OpenCode guidance remains thin and repo-local or copy/paste-oriented."
    - "Generated guidance tells agents to pass explicit `repo`, call `prepare_task` first, read files with native tools, and call `get_pack` progressively."
    - "Generated guidance explains that MCP pack resources are session-scoped and `get_pack` is the durable reconnect-safe path."
    - "Generated guidance does not mutate global agent configuration by default and does not inject large static repository context."
  artifacts:
    - path: "crates/ctxpack-core/src/init.rs"
      provides: "Adapter template text plus regression tests"
  key_links:
    - from: "crates/ctxpack-core/src/init.rs"
      to: "generated AGENTS, Cursor, Claude, OpenCode, and Codex setup guidance"
      via: "adapter_content, adapter_files, and InitReport codex_mcp_setup"
      pattern: "prepare_task|get_pack|session-scoped|repo"
---

<objective>
Refresh generated agent setup guidance for first-pack adoption.

Purpose: Users should be able to wire ctxpack into Codex, Claude Code, Cursor, and OpenCode without hidden global mutations or stale static context.
Output: Thin adapter/setup text that encodes the current MCP-first flow and guards against large static instructions.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/research/FEATURES.md
@.planning/research/PITFALLS.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-CONTEXT.md
@.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-02-PLAN.md
@.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-02-SUMMARY.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-01-SUMMARY.md
@crates/ctxpack-core/src/init.rs

<decision_trace>
- ADPT-02: Generated Codex, Claude Code, Cursor, and OpenCode setup guidance must stay thin, repo-local or copy/paste-oriented, and must not mutate global config by default.
- ADPT-03: Guidance must include explicit `repo`, progressive `prepare_task` -> native file reads -> `get_pack`, and pack-resource session-scope caveats.
- Phase 6 deferred ideas: no automatic global agent config writes and no Cursor/OpenCode real-client tool-call claims without machine-checkable proof.
</decision_trace>

<interfaces>
From `crates/ctxpack-core/src/init.rs`:
```rust
pub const AGENTS_SECTION: &str;
pub const CURSOR_RULE: &str;
pub const CLAUDE_BUGFIX_COMMAND: &str;
pub const CLAUDE_MCP_SNIPPET: &str;
pub const OPENCODE_SNIPPET: &str;
pub const CODEX_MCP_SETUP: &str;
pub fn adapter_path(adapter: AgentAdapter) -> &'static str;
pub fn adapter_content(adapter: AgentAdapter) -> &'static str;
pub fn adapter_files(adapter: AgentAdapter) -> Vec<(&'static str, &'static str)>;
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Tighten adapter guidance regression tests</name>
  <files>crates/ctxpack-core/src/init.rs</files>
  <behavior>
    - Test 1: AGENTS, Cursor, Claude command, OpenCode, and Codex guidance each contain `prepare_task`, `get_pack`, explicit `repo`, and wording that direct file reads happen with the agent's native tools before editing.
    - Test 2: AGENTS, Cursor, Claude command, OpenCode, and Codex guidance each mention pack resource session scope or same-session behavior, and point to `get_pack` when reconnecting or needing durable materialization.
    - Test 3: Codex guidance contains a copy/paste setup command such as `codex mcp add ctxpack -- ctxpack serve-mcp` or an equivalent config snippet, and explicitly says ctxpack does not apply it automatically.
    - Test 4: Claude guidance stays project-local or mergeable and `.ctxpack/adapters/claude-mcp.json` remains valid JSON.
    - Test 5: Cursor/OpenCode artifacts remain under repo-local paths, stay below existing byte limits, and do not contain static dump phrases such as `Repository map`, `inventory dump`, `source snippets`, or `paste this repo`.
  </behavior>
  <action>Extend the existing adapter tests from Phase 4 to cover the full ADPT-02 and ADPT-03 contract. Keep tests local to `crates/ctxpack-core/src/init.rs`; do not add external client invocations here. Use helper iteration over generated guidance artifacts so all agents get the same thinness, repo-explicit, progressive-pack, and session-scope checks. Preserve existing JSON parse tests for Claude and OpenCode snippets.</action>
  <verify>
    <automated>cargo test -p ctxpack-core adapter -- --nocapture</automated>
  </verify>
  <done>Tests fail if generated setup guidance loses explicit repo usage, progressive pack behavior, session-scope caveats, or the no-global-mutation boundary.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Update generated Codex, Claude, Cursor, and OpenCode setup text</name>
  <files>crates/ctxpack-core/src/init.rs</files>
  <behavior>
    - Test 1: Generated guidance tells agents to call `prepare_task` with the active repository path as `repo` before non-trivial planning.
    - Test 2: Generated guidance tells agents to read target files natively before editing and to use `get_pack` progressively only when direct file reads or brief context are insufficient.
    - Test 3: Generated guidance explains pack resources returned by `prepare_task` are same-session MCP resources; after reconnect or restart, call `get_pack`.
    - Test 4: Codex, Claude, Cursor, and OpenCode setup text remains concise, repo-local or copy/paste-oriented, and does not promise real-client proof beyond deterministic protocol setup.
  </behavior>
  <action>Refresh `AGENTS_SECTION`, `CURSOR_RULE`, `CLAUDE_BUGFIX_COMMAND`, `OPENCODE_SNIPPET`, and `CODEX_MCP_SETUP` to satisfy ADPT-02 and ADPT-03. Keep snippets small and dynamic; do not embed repo file lists, generated cards, source examples, or large context. Add absolute-binary troubleshooting guidance only as concise text such as "if the agent cannot spawn `ctxpack`, replace it with the absolute path from `which ctxpack`"; do not attempt to resolve a user-specific path in generated repo files.</action>
  <verify>
    <automated>cargo test -p ctxpack-core adapter -- --nocapture && cargo test -p ctxpack-core init -- --nocapture</automated>
  </verify>
  <done>Generated setup guidance is current, thin, repo-explicit, progressive, session-scope-aware, and still only writes repo-local artifacts.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack-core adapter -- --nocapture`
- `cargo test -p ctxpack-core init -- --nocapture`
- `cargo test --workspace`
</verification>

<success_criteria>
Plan 02 is complete when generated agent setup artifacts satisfy ADPT-02 and ADPT-03 without adding global config mutation, static repo context, new MCP tools, or client-proof claims not backed by machine-checkable smoke.
</success_criteria>

<output>
After completion, create `.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-02-SUMMARY.md`
</output>
