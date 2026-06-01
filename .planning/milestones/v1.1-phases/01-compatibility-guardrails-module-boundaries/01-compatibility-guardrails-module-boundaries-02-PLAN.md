---
phase: 01-compatibility-guardrails-module-boundaries
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/ctxhelm-core/src/contracts.rs
  - crates/ctxhelm-compiler/src/lib.rs
  - crates/ctxhelm-mcp/src/lib.rs
autonomous: true
requirements: [CONT-02, CONT-03]
must_haves:
  truths:
    - "Maintainer can detect drift in public JSON contracts before internal changes land."
    - "Maintainer can detect drift in MCP tools, resources, prompts, structuredContent, text fallback, and error responses."
    - "Current MCP pack-resource behavior is characterized as session-scoped without changing durability semantics."
  artifacts:
    - path: "crates/ctxhelm-core/src/contracts.rs"
      provides: "Core contract shape tests"
      contains: "public_contract_shapes"
    - path: "crates/ctxhelm-compiler/src/lib.rs"
      provides: "Compiler report and pack contract tests"
      contains: "historical_eval_report"
    - path: "crates/ctxhelm-mcp/src/lib.rs"
      provides: "MCP compatibility tests"
      contains: "IMPLEMENTED_MCP_TOOL_NAMES"
  key_links:
    - from: "crates/ctxhelm-mcp/src/lib.rs"
      to: "crates/ctxhelm-core/src/contracts.rs"
      via: "structuredContent serializes ContextPlan and ContextPack"
      pattern: "structuredContent"
    - from: "crates/ctxhelm-compiler/src/lib.rs"
      to: "crates/ctxhelm-core/src/contracts.rs"
      via: "HistoricalEvalReport and ContextPack field-shape tests"
      pattern: "serde_json::to_value"
---

<objective>
Lock public JSON and MCP protocol compatibility before module splitting.

Purpose: CONT-02 and CONT-03 plus decisions D-02, D-03, D-07, D-08, and D-09 require stable contract-shape tests for CLI/MCP consumers and public serde structures.
Output: Focused contract tests in core/compiler and comprehensive MCP compatibility tests in the MCP crate.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/STATE.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-RESEARCH.md
@.planning/codebase/ARCHITECTURE.md
@.planning/codebase/TESTING.md
@.planning/codebase/CONVENTIONS.md
@AGENTS.md
@crates/ctxhelm-core/src/contracts.rs
@crates/ctxhelm-compiler/src/lib.rs
@crates/ctxhelm-mcp/src/lib.rs

<interfaces>
Public compatibility surfaces from D-03: `ContextPlan`, `ContextPack`, `EvalTrace`, `HistoricalEvalReport`, MCP `structuredContent`, and major CLI JSON outputs.

MCP implemented tools must remain exactly:
```text
prepare_task
search
related
get_pack
related_tests
current_diff
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend core and compiler JSON shape tests</name>
  <files>crates/ctxhelm-core/src/contracts.rs, crates/ctxhelm-compiler/src/lib.rs</files>
  <read_first>
    - crates/ctxhelm-core/src/contracts.rs
    - crates/ctxhelm-compiler/src/lib.rs
    - crates/ctxhelm/src/main.rs
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-RESEARCH.md
  </read_first>
  <behavior>
    - `ContextPlan` serialized JSON contains camelCase fields and does not contain snake_case aliases.
    - `ContextPack` serialized JSON contains source-free provenance fields `repoId`, `taskHash`, `targetAgent`, `budget`, `sections`, `tokenEstimate`, and `privacyStatus`.
    - `EvalTrace` serialized JSON contains `sourceTextLogged: false` and does not contain source snippets or prompt text.
    - `HistoricalEvalReport` serialized JSON contains source-free report fields used by `eval history`.
  </behavior>
  <action>
    In `crates/ctxhelm-core/src/contracts.rs`, add or extend tests named `context_plan_public_json_shape_is_stable`, `context_pack_public_json_shape_is_stable`, and `eval_trace_public_json_shape_is_source_free`. Assert exact field presence with `serde_json::to_value` and `Value::as_object().unwrap().contains_key(...)` for current public fields. Assert snake_case aliases such as `task_id`, `target_files`, `related_tests`, `repo_id`, `task_hash`, `target_agent`, and `source_text_logged` are absent. In `crates/ctxhelm-compiler/src/lib.rs`, add or extend `historical_eval_report_public_json_shape_is_stable` to assert fields `mode`, `limit`, `evaluatedCommits`, `fileRecallAt5`, `fileRecallAt10`, `sourceRecallAt5`, `testRecallAt5`, `lexicalBaselineRecallAt5`, `testRecommendationRate`, `averageRecommendedContextFiles`, `topMissingFiles`, and `privacyStatus`. Do not introduce schema generation or new public fields unless tests document additive compatibility.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-core public_json_shape -- --nocapture</automated>
    <automated>cargo test -p ctxhelm-compiler public_json_shape -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `rg -n 'context_plan_public_json_shape_is_stable|context_pack_public_json_shape_is_stable|eval_trace_public_json_shape_is_source_free' crates/ctxhelm-core/src/contracts.rs` finds all three tests.
    - `rg -n 'historical_eval_report_public_json_shape_is_stable|evaluatedCommits|fileRecallAt5|lexicalBaselineRecallAt5|topMissingFiles|privacyStatus' crates/ctxhelm-compiler/src/lib.rs` finds all report fields.
    - `cargo test -p ctxhelm-core public_json_shape -- --nocapture` passes.
    - `cargo test -p ctxhelm-compiler public_json_shape -- --nocapture` passes.
  </acceptance_criteria>
  <done>Public JSON contract drift is guarded for core contracts and historical eval reports.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Harden MCP compatibility tests for protocol surfaces</name>
  <files>crates/ctxhelm-mcp/src/lib.rs</files>
  <read_first>
    - crates/ctxhelm-mcp/src/lib.rs
    - crates/ctxhelm-core/src/contracts.rs
    - crates/ctxhelm-compiler/src/lib.rs
    - crates/ctxhelm-index/src/lib.rs
    - .planning/codebase/ARCHITECTURE.md
    - .planning/codebase/TESTING.md
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
  </read_first>
  <behavior>
    - `initialize` returns `serverInfo.name == "ctxhelm"` and tools/resources/prompts capabilities with `listChanged: false`.
    - `tools/list` returns exactly `prepare_task`, `search`, `related`, `get_pack`, `related_tests`, and `current_diff` in that order.
    - Changed tool tests assert both `content[0].text` and the current per-tool `structuredContent` shape: object for `prepare_task`, `search`, `related`, `get_pack`, and `current_diff`; array for `related_tests`.
    - `resources/list` asserts the current static list only: `ctxhelm://repo/summary`, `ctxhelm://repo/test-map`, `ctxhelm://repo/dependency-graph`, and `ctxhelm://pack/guide`.
    - `resources/read` covers static repo resources plus dynamic `ctxhelm://file/...`, `ctxhelm://symbol/...`, and same-session generated `ctxhelm://pack/...` reads.
    - `prompts/list` and `prompts/get` preserve prompt names and workflow prompt shape.
    - Unknown methods and invalid params preserve JSON-RPC error codes already used by the crate.
  </behavior>
  <action>
    Extend the existing `#[cfg(test)] mod tests` in `crates/ctxhelm-mcp/src/lib.rs`. Add tests named `initialize_public_capabilities_are_stable`, `tools_list_public_surface_is_exact`, `tool_calls_include_text_and_structured_content`, `resources_public_uri_shapes_are_stable`, `prompts_public_surface_is_stable`, `pack_resources_are_session_scoped_characterization`, and `json_rpc_error_codes_are_stable`. Use the existing `handle_line`, `run_server`, `fixture_repo`, and JSON-RPC request style.

    Per D-07, assert `IMPLEMENTED_MCP_TOOL_NAMES == ["prepare_task", "search", "related", "get_pack", "related_tests", "current_diff"]`.

    Per D-08, assert `content[0].text` is a string for every changed tool, assert object-shaped `structuredContent` for `prepare_task`, `search`, `related`, `get_pack`, and `current_diff`, and assert array-shaped `structuredContent` for `related_tests` using the current `response["result"]["structuredContent"][0]["path"]` indexing pattern.

    Per D-09, assert `resources/list` returns only the current static descriptors: `ctxhelm://repo/summary`, `ctxhelm://repo/test-map`, `ctxhelm://repo/dependency-graph`, and `ctxhelm://pack/guide`.

    Use `resources/read` tests to characterize repo resources, dynamic `ctxhelm://file/...` resources, dynamic `ctxhelm://symbol/...` resources, and generated `ctxhelm://pack/...` resources available only within the same server process; do not implement cross-process durability.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-mcp public_surface -- --nocapture</automated>
    <automated>cargo test -p ctxhelm-mcp session_scoped -- --nocapture</automated>
    <automated>cargo test -p ctxhelm-mcp error_codes -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `rg -n 'initialize_public_capabilities_are_stable|tools_list_public_surface_is_exact|tool_calls_include_text_and_structured_content|resources_public_uri_shapes_are_stable|prompts_public_surface_is_stable|pack_resources_are_session_scoped_characterization|json_rpc_error_codes_are_stable' crates/ctxhelm-mcp/src/lib.rs` finds all seven tests.
    - `rg -n 'prepare_task.*search.*related.*get_pack.*related_tests.*current_diff|structuredContent.*\\[0\\]|content\\]\\[0\\]|ctxhelm://repo/summary|ctxhelm://repo/test-map|ctxhelm://repo/dependency-graph|ctxhelm://pack/guide|ctxhelm://file|ctxhelm://symbol|ctxhelm://pack|listChanged' crates/ctxhelm-mcp/src/lib.rs` finds the public-surface assertions.
    - `cargo test -p ctxhelm-mcp public_surface -- --nocapture` passes.
    - `cargo test -p ctxhelm-mcp session_scoped -- --nocapture` passes.
    - `cargo test -p ctxhelm-mcp error_codes -- --nocapture` passes.
  </acceptance_criteria>
  <done>MCP compatibility is characterized across tools, resources, prompts, session cache behavior, structuredContent, text fallback, and errors.</done>
</task>

</tasks>

<verification>
Run the focused commands above, then run `cargo test -p ctxhelm-core -p ctxhelm-compiler -p ctxhelm-mcp`.
</verification>

<success_criteria>
CONT-02 and CONT-03 are satisfied when contract tests fail on public JSON or MCP drift and still pass on unchanged current behavior.
</success_criteria>

<output>
After completion, create `.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-02-SUMMARY.md`.
</output>
