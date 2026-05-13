---
phase: 04-agent-native-client-durability
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/ctxpack-core/src/init.rs
autonomous: true
requirements: [AGNT-04]
must_haves:
  truths:
    - "Generated AGENTS, Cursor, Claude, OpenCode, and Codex guidance directs agents to dynamic ctxpack calls."
    - "Adapter guidance tells agents to pass explicit repo when known and load get_pack progressively only when needed."
    - "Generated guidance remains concise and does not embed static repo maps, inventories, source snippets, or context dumps."
  artifacts:
    - path: "crates/ctxpack-core/src/init.rs"
      provides: "Thin dynamic adapter constants plus snapshot/size tests"
  key_links:
    - from: "crates/ctxpack-core/src/init.rs"
      to: "generated AGENTS.md and native adapter files"
      via: "run_init adapter_content and adapter_files"
      pattern: "prepare_task|get_pack|repo"
---

<objective>
Keep generated agent-native guidance thin, dynamic, and repo-explicit.

Purpose: Phase 4 should make agents call ctxpack at runtime, not bake stale repository context into rule files.
Output: Adapter wording and tests that prevent static context dumps or outdated prepare_task-only guidance.
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
@crates/ctxpack-core/src/init.rs

<decision_trace>
- D-04: Generated adapter guidance must stay thin, dynamic, repo-explicit, and avoid static context dumps.
- D-05: Do not add autonomous editing behavior, cloud/vector scope, or new daily app surfaces.
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
pub fn adapter_content(adapter: AgentAdapter) -> &'static str;
pub fn adapter_files(adapter: AgentAdapter) -> Vec<(&'static str, &'static str)>;
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add adapter thinness and dynamic-guidance guards</name>
  <files>crates/ctxpack-core/src/init.rs</files>
  <behavior>
    - Test 1: AGENTS, Cursor, Claude, OpenCode, and Codex guidance rejects static-dump phrases such as `Repository map:`, `Full context pack`, `inventory dump`, `paste this repo`, and `source snippets`.
    - Test 2: Each generated guidance artifact stays below a small size limit: AGENTS <= 1400 bytes, Cursor <= 1400, Claude command <= 1400, OpenCode snippet <= 900, Codex setup <= 900.
    - Test 3: Existing guidance still mentions dynamic `prepare_task` and passing the active repository path as `repo` when known.
  </behavior>
  <action>Add tests in the existing `#[cfg(test)]` module per D-04. Use a small helper that iterates over the constants and asserts explicit `repo`, forbidden static-dump phrases, and byte-size limits. Keep Task 1 compatible with the current prepare-task-oriented wording; assertions that require `get_pack` or progressive loading belong in Task 2 after the wording is refreshed. Keep the tests focused on generated guidance only; do not modify runtime MCP behavior or add external dependencies.</action>
  <verify>
    <automated>cargo test -p ctxpack-core adapter -- --nocapture</automated>
  </verify>
  <done>Adapter tests fail if generated guidance grows into static context dumps or omits dynamic repo-explicit ctxpack usage.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Refresh adapter wording for prepare_task and get_pack</name>
  <files>crates/ctxpack-core/src/init.rs</files>
  <behavior>
    - Test 1: OpenCode and Codex text no longer says `The first implemented tool is prepare_task`.
    - Test 2: AGENTS and native adapters instruct agents to call `prepare_task` first and request `get_pack` only when direct file reads or brief context are insufficient.
    - Test 3: Claude MCP snippet remains a small local stdio server snippet and does not mutate global client configuration.
    - Test 4: AGENTS, Cursor, Claude, OpenCode, and Codex guidance each mention dynamic `prepare_task` and either `get_pack` or progressive pack loading.
  </behavior>
  <action>Update `AGENTS_SECTION`, `CURSOR_RULE`, `CLAUDE_BUGFIX_COMMAND`, `OPENCODE_SNIPPET`, and `CODEX_MCP_SETUP` to reflect the current small MCP surface. The wording must tell agents to pass explicit `repo`, read actual files with native tools before editing, use `get_pack` progressively instead of static dumps, and keep ctxpack read-only. Preserve existing generated file paths and JSON validity for snippets. Do not include repo file lists, generated cards inline, or source examples.</action>
  <verify>
    <automated>cargo test -p ctxpack-core adapter -- --nocapture && cargo test -p ctxpack-core init -- --nocapture</automated>
  </verify>
  <done>Generated adapter content is current, concise, dynamic, repo-explicit, and covered by regression tests.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack-core adapter -- --nocapture`
- `cargo test -p ctxpack-core init -- --nocapture`
- `cargo test --workspace`
</verification>

<success_criteria>
Plan 02 is complete when adapter generation cannot regress into large static context, outdated prepare_task-only claims, or ambiguous repo guidance.
</success_criteria>

<output>
After completion, create `.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-02-SUMMARY.md`
</output>
