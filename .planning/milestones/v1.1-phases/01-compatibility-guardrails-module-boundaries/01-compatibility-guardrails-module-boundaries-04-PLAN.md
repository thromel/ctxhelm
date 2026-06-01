---
phase: 01-compatibility-guardrails-module-boundaries
plan: 04
type: execute
wave: 2
depends_on:
  - 01-compatibility-guardrails-module-boundaries-01
  - 01-compatibility-guardrails-module-boundaries-02
files_modified:
  - crates/ctxhelm-compiler/src/lib.rs
  - crates/ctxhelm-compiler/src/planning.rs
  - crates/ctxhelm-compiler/src/packs.rs
  - crates/ctxhelm-compiler/src/cards.rs
  - crates/ctxhelm-compiler/src/eval.rs
  - crates/ctxhelm-mcp/src/lib.rs
  - crates/ctxhelm-mcp/src/protocol.rs
  - crates/ctxhelm-mcp/src/schemas.rs
  - crates/ctxhelm-mcp/src/tools.rs
  - crates/ctxhelm-mcp/src/resources.rs
  - crates/ctxhelm-mcp/src/prompts.rs
autonomous: true
requirements: [CONT-04]
must_haves:
  truths:
    - "Maintainer can split compiler internals while public planning, pack, card, render, and eval APIs remain stable."
    - "Maintainer can split MCP internals while tool names, resource URI shapes, prompt names, structuredContent, text fallback, and session cache behavior remain stable."
    - "Workspace tests and Wave 1 compatibility guardrails pass after compiler and MCP splits."
  artifacts:
    - path: "crates/ctxhelm-compiler/src/lib.rs"
      provides: "Stable compiler crate facade"
      contains: "pub use"
    - path: "crates/ctxhelm-mcp/src/lib.rs"
      provides: "Stable MCP crate facade"
      contains: "pub fn run_stdio_server"
    - path: "crates/ctxhelm-mcp/src/tools.rs"
      provides: "MCP tool handlers"
      contains: "prepare_task"
  key_links:
    - from: "crates/ctxhelm/src/main.rs"
      to: "crates/ctxhelm-compiler/src/lib.rs"
      via: "unchanged compiler public functions"
      pattern: "compile_context_pack_with_plan_and_paths_for_agent"
    - from: "crates/ctxhelm/src/main.rs"
      to: "crates/ctxhelm-mcp/src/lib.rs"
      via: "serve-mcp calls run_stdio_server"
      pattern: "run_stdio_server"
---

<objective>
Split `ctxhelm-compiler` and `ctxhelm-mcp` into focused modules behind stable crate-root facades.

Purpose: CONT-04 plus decisions D-10, D-11, and D-12 require module boundaries by current concern while preserving CLI, MCP, and library behavior.
Output: Focused compiler and MCP modules with unchanged public exports, staged compiler/MCP validation, and passing compatibility guardrails.
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
@.planning/codebase/STRUCTURE.md
@.planning/codebase/TESTING.md
@.planning/codebase/CONVENTIONS.md
@AGENTS.md
@crates/ctxhelm-compiler/src/lib.rs
@crates/ctxhelm-mcp/src/lib.rs
@crates/ctxhelm/src/main.rs

<interfaces>
Compiler public APIs must remain available from `ctxhelm_compiler`: `HistoricalEvalOptions`, `HistoricalEvalReport`, `HistoricalMissingFileSummary`, `HistoricalCommitEval`, `ContextCardsOptions`, `GeneratedContextCard`, `ContextCardsReport`, `empty_plan_for_task`, `prepare_context_plan`, `prepare_context_plan_with_paths`, `compile_context_pack`, `compile_context_pack_with_plan`, `compile_context_pack_with_plan_and_paths`, `compile_context_pack_with_plan_for_agent`, `compile_context_pack_with_plan_and_paths_for_agent`, `compile_context_pack_from_plan`, `compile_context_pack_from_plan_for_agent`, `render_pack_markdown`, `eval_trace_for_plan`, `eval_trace_for_pack`, `evaluate_historical_commits`, and `generate_context_cards`.

MCP public API must continue to expose `PLANNED_MCP_TOOL_NAMES`, `IMPLEMENTED_MCP_TOOL_NAMES`, `run_stdio_server`, and `run_server` from `ctxhelm_mcp`.

<execution_staging>
This plan intentionally remains one plan because both refactors depend on the same Wave 1 compatibility guardrails, but execution must be staged to control the 11-file blast radius:

1. Stage A: Complete Task 1 compiler-only split and run every Task 1 verification command before editing `crates/ctxhelm-mcp/src/*`.
2. Stage B: Complete Task 2 MCP-only split after Stage A passes and run every Task 2 verification command before final workspace validation.
3. Stage C: Run Task 3 final compatibility validation only after Stages A and B independently pass.

If Stage A fails, fix only compiler files before continuing. If Stage B fails, fix only MCP files unless the failure proves an exported compiler facade regression from Stage A.
</execution_staging>
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Split compiler planning, packs, cards, and eval modules</name>
  <files>crates/ctxhelm-compiler/src/lib.rs, crates/ctxhelm-compiler/src/planning.rs, crates/ctxhelm-compiler/src/packs.rs, crates/ctxhelm-compiler/src/cards.rs, crates/ctxhelm-compiler/src/eval.rs</files>
  <read_first>
    - crates/ctxhelm-compiler/src/lib.rs
    - crates/ctxhelm-core/src/contracts.rs
    - crates/ctxhelm-index/src/lib.rs
    - crates/ctxhelm/src/main.rs
    - crates/ctxhelm-mcp/src/lib.rs
    - crates/ctxhelm/tests/cli_compat.rs
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
  </read_first>
  <behavior>
    - Compiler public APIs remain import-compatible from the crate root.
    - Plan, pack, card, render, trace, and historical eval tests keep passing.
    - JSON contract tests from Plan 02 still pass.
  </behavior>
  <action>
    Stage A only: complete this compiler split and run the Task 1 verification commands before editing MCP files. Move context-plan preparation, ranking, anchor normalization, and confidence helpers into `crates/ctxhelm-compiler/src/planning.rs`. Move pack compilation, pack limits, snippet rendering, final checklist rendering, and `render_pack_markdown` into `crates/ctxhelm-compiler/src/packs.rs`. Move context-card generation and card render helpers into `crates/ctxhelm-compiler/src/cards.rs`. Move historical eval structs/functions, source-free metrics, worktree extraction helpers, report helpers, and eval trace helpers into `crates/ctxhelm-compiler/src/eval.rs`. Replace moved sections in `lib.rs` with `mod planning; mod packs; mod cards; mod eval;` plus `pub use` facade exports for every public API listed in this plan. Preserve serde rename rules, Markdown headings, provenance strings, pack section kinds, eval metrics, source-free guarantees, and behavior of low-information task detection. Do not add retrieval behavior or metric expansion in this phase.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-compiler</automated>
    <automated>cargo test -p ctxhelm-core public_json_shape -- --nocapture</automated>
    <automated>cargo test -p ctxhelm-compiler public_json_shape -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `test -f crates/ctxhelm-compiler/src/planning.rs && test -f crates/ctxhelm-compiler/src/packs.rs && test -f crates/ctxhelm-compiler/src/cards.rs && test -f crates/ctxhelm-compiler/src/eval.rs` succeeds.
    - `rg -n '^mod planning;|^mod packs;|^mod cards;|^mod eval;|pub use planning::|pub use packs::|pub use cards::|pub use eval::' crates/ctxhelm-compiler/src/lib.rs` finds facade declarations.
    - `rg -n 'pub fn prepare_context_plan_with_paths|pub fn compile_context_pack_with_plan_and_paths_for_agent|pub fn render_pack_markdown|pub fn generate_context_cards|pub fn evaluate_historical_commits|pub fn eval_trace_for_plan' crates/ctxhelm-compiler/src/*.rs` finds public APIs in modules or facade.
    - `cargo test -p ctxhelm-compiler` passes.
  </acceptance_criteria>
  <done>`ctxhelm-compiler` is split by planning, packs, cards, and eval with stable public exports.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Split MCP protocol, schema, tool, resource, and prompt modules</name>
  <files>crates/ctxhelm-mcp/src/lib.rs, crates/ctxhelm-mcp/src/protocol.rs, crates/ctxhelm-mcp/src/schemas.rs, crates/ctxhelm-mcp/src/tools.rs, crates/ctxhelm-mcp/src/resources.rs, crates/ctxhelm-mcp/src/prompts.rs</files>
  <read_first>
    - crates/ctxhelm-mcp/src/lib.rs
    - crates/ctxhelm-core/src/contracts.rs
    - crates/ctxhelm-compiler/src/lib.rs
    - crates/ctxhelm-index/src/lib.rs
    - crates/ctxhelm/src/main.rs
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
  </read_first>
  <behavior>
    - `run_stdio_server`, `run_server`, `PLANNED_MCP_TOOL_NAMES`, and `IMPLEMENTED_MCP_TOOL_NAMES` remain exported from `ctxhelm_mcp`.
    - MCP compatibility tests from Plan 02 pass unchanged.
    - Tool names, resource URI shapes, prompt names, text fallback, structuredContent, and session pack cache behavior remain unchanged.
  </behavior>
  <action>
    Stage B only: start this MCP split after Task 1 passes, then run the Task 2 verification commands before final workspace validation. Move JSON-RPC request/response types, `RpcError`, `run_stdio_server`, `run_server`, `handle_line`, and `handle_request` into `crates/ctxhelm-mcp/src/protocol.rs` while re-exporting `run_stdio_server` and `run_server` from `lib.rs`. Move `initialize_result`, `tools_list_result`, tool input schema builders, `resources_list_result`, `prompts_list_result`, and descriptor helpers into `crates/ctxhelm-mcp/src/schemas.rs`. Move `call_tool`, all `call_*` tool handlers, argument structs, anchor helpers, `tool_json_result`, and bounded limits into `crates/ctxhelm-mcp/src/tools.rs`. Move resource reads, repo summary/test/dependency resources, file/symbol resources, pack resource cache, and pack guide into `crates/ctxhelm-mcp/src/resources.rs`. Move prompt handlers and `workflow_prompt` into `crates/ctxhelm-mcp/src/prompts.rs`. Keep module visibility `pub(crate)` unless an item is part of the public crate API. Do not change `MCP_PROTOCOL_VERSION`, `JSONRPC_VERSION`, tool order, resource URI strings, prompt names, pack cache semantics, or error codes.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-mcp</automated>
    <automated>cargo test -p ctxhelm --test cli_compat serve_mcp -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `test -f crates/ctxhelm-mcp/src/protocol.rs && test -f crates/ctxhelm-mcp/src/schemas.rs && test -f crates/ctxhelm-mcp/src/tools.rs && test -f crates/ctxhelm-mcp/src/resources.rs && test -f crates/ctxhelm-mcp/src/prompts.rs` succeeds.
    - `rg -n '^mod protocol;|^mod schemas;|^mod tools;|^mod resources;|^mod prompts;|pub use protocol::\\{run_server, run_stdio_server\\}' crates/ctxhelm-mcp/src/lib.rs` finds facade declarations.
    - `rg -n 'IMPLEMENTED_MCP_TOOL_NAMES|prepare_task|search|related|get_pack|related_tests|current_diff|structuredContent|ctxhelm://pack|workflow_prompt|method_not_found' crates/ctxhelm-mcp/src/*.rs` finds stable protocol markers.
    - `cargo test -p ctxhelm-mcp` passes.
    - `cargo test -p ctxhelm --test cli_compat serve_mcp -- --nocapture` passes.
  </acceptance_criteria>
  <done>`ctxhelm-mcp` is split by protocol, schemas, tools, resources, and prompts with stable public behavior.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Run full compatibility validation after module splits</name>
  <files>crates/ctxhelm-compiler/src/lib.rs, crates/ctxhelm-mcp/src/lib.rs, crates/ctxhelm/tests/cli_compat.rs</files>
  <read_first>
    - crates/ctxhelm-compiler/src/lib.rs
    - crates/ctxhelm-mcp/src/lib.rs
    - crates/ctxhelm/tests/cli_compat.rs
    - AGENTS.md
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
  </read_first>
  <behavior>
    - Workspace tests pass.
    - CLI help smoke passes after CLI-related compatibility work.
    - Public contract and MCP guardrails from Wave 1 still pass.
  </behavior>
  <action>
    Stage C only: run the required validation commands after the compiler split and MCP split have each passed their own focused verification. If a failure shows public drift, restore the previous public field/tool/resource/prompt/error behavior and keep the module split internal. If a failure exposes an actual pre-existing bug, add a failing characterization/regression test first and keep any fix narrowly scoped per D-12. Do not update README behavior, introduce new retrieval ranking, add freshness/privacy diagnostics, add real-client smoke scripts, or change pack-resource durability in this phase.
  </action>
  <verify>
    <automated>cargo test --workspace</automated>
    <automated>cargo run -p ctxhelm -- --help</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test --workspace` passes.
    - `cargo run -p ctxhelm -- --help` exits 0 and lists `serve-mcp`.
    - `cargo test -p ctxhelm --test cli_compat` passes.
    - `cargo test -p ctxhelm-mcp` passes.
  </acceptance_criteria>
  <done>Compiler and MCP splits preserve all public behavior under full validation.</done>
</task>

</tasks>

<verification>
Run `cargo test --workspace`, `cargo run -p ctxhelm -- --help`, `cargo test -p ctxhelm --test cli_compat`, and `cargo test -p ctxhelm-mcp`.
</verification>

<success_criteria>
CONT-04 is satisfied for compiler and MCP when both crates are split behind stable facades and all guardrail tests pass without public contract drift.
</success_criteria>

<output>
After completion, create `.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-04-SUMMARY.md`.
</output>
