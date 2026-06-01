---
phase: 02-trust-layer-operational-diagnostics
plan: 05
type: execute
wave: 5
depends_on: ["02-04"]
files_modified:
  - crates/ctxhelm-index/src/traces.rs
  - crates/ctxhelm/src/main.rs
  - crates/ctxhelm/tests/cli_compat.rs
  - crates/ctxhelm/tests/common/mod.rs
  - crates/ctxhelm-mcp/src/tools.rs
  - crates/ctxhelm-mcp/src/resources.rs
  - crates/ctxhelm-mcp/src/schemas.rs
  - crates/ctxhelm-mcp/src/lib.rs
autonomous: true
requirements: [SAFE-01, SAFE-02, SAFE-04, SAFE-05, SAFE-06, DIAG-01, DIAG-02, DIAG-04]
must_haves:
  truths:
    - "CLI and MCP read-oriented outputs expose diagnostics in stable structured fields while preserving existing compatibility shapes."
    - "Trace writes are visible, controllable, and non-fatal for prepare-task/get-pack style read operations."
    - "MCP file resources revalidate source paths with the same current safe-inventory policy used by packs."
    - "Full workspace validation and CLI help pass after Phase 2 wiring."
  artifacts:
    - path: "crates/ctxhelm/src/main.rs"
      provides: "CLI diagnostics projection and trace controls"
      contains: "no_trace"
    - path: "crates/ctxhelm-index/src/traces.rs"
      provides: "Non-fatal trace append status helper"
      contains: "try_append_eval_trace"
    - path: "crates/ctxhelm-mcp/src/tools.rs"
      provides: "MCP structuredContent diagnostics and trace controls"
      contains: "diagnostics"
    - path: "crates/ctxhelm-mcp/src/resources.rs"
      provides: "Fresh, policy-gated file resources"
      contains: "read_safe_source"
    - path: "crates/ctxhelm/tests/cli_compat.rs"
      provides: "Binary-level diagnostics and constrained-home guardrails"
      contains: "diagnostics"
  key_links:
    - from: "crates/ctxhelm/src/main.rs"
      to: "crates/ctxhelm-index/src/traces.rs"
      via: "non-fatal trace append for read commands"
      pattern: "try_append_eval_trace"
    - from: "crates/ctxhelm-mcp/src/resources.rs"
      to: "crates/ctxhelm-index/src/policy.rs"
      via: "safe file-resource reads"
      pattern: "read_safe_source"
    - from: "crates/ctxhelm-mcp/src/tools.rs"
      to: "crates/ctxhelm-core/src/contracts.rs"
      via: "structuredContent diagnostics"
      pattern: "diagnostics"
---

<objective>
Expose Phase 2 diagnostics and constrained-write behavior at the CLI and MCP boundaries.

Purpose: DIAG-02 and SAFE-06 are only satisfied when users and agents can see diagnostics in public outputs and read-oriented commands remain usable when trace/cache writes fail. This final plan wires the trust layer into the product surfaces without changing the small MCP tool surface or adding Phase 3 retrieval work.
Output: CLI/MCP diagnostics fields, non-fatal trace write status, revalidated MCP file resources, compatibility tests, and full validation.
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
@.planning/phases/02-trust-layer-operational-diagnostics/02-CONTEXT.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-RESEARCH.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-01-SUMMARY.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-02-SUMMARY.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-03-SUMMARY.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-04-SUMMARY.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-01-SUMMARY.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-02-SUMMARY.md
@.planning/codebase/ARCHITECTURE.md
@.planning/codebase/TESTING.md
@.planning/codebase/CONVENTIONS.md
@AGENTS.md

<interfaces>
Existing CLI/MCP compatibility surfaces to preserve:
```rust
pub const IMPLEMENTED_MCP_TOOL_NAMES: &[&str] = &[
    "prepare_task", "search", "related", "get_pack", "related_tests", "current_diff",
];
pub fn run_stdio_server() -> io::Result<()>;
```

Phase 4 compiler outputs now carry `diagnostics` on plans and packs.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Make CLI diagnostics and trace writes operationally safe</name>
  <files>crates/ctxhelm-index/src/traces.rs, crates/ctxhelm/src/main.rs, crates/ctxhelm/tests/cli_compat.rs, crates/ctxhelm/tests/common/mod.rs</files>
  <read_first>
    - `crates/ctxhelm/src/main.rs`
    - `crates/ctxhelm-index/src/traces.rs`
    - `crates/ctxhelm/tests/cli_compat.rs`
    - `crates/ctxhelm/tests/common/mod.rs`
  </read_first>
  <behavior>
    - SAFE-06: `prepare-task` and `get-pack` continue returning context when trace append fails, with a `trace_write_failed` diagnostic.
    - SAFE-06: users can disable trace recording on read-oriented commands with an additive CLI control such as `--no-trace`.
    - DIAG-02: CLI JSON object outputs for plan/pack include `diagnostics`; existing array-shaped commands keep compatibility unless an explicit diagnostics mode is added.
    - DIAG-04: binary CLI tests cover stale-cache diagnostics and constrained `CTXHELM_HOME`.
  </behavior>
  <action>
    Add a non-fatal trace helper such as `try_append_eval_trace` in `traces.rs` that returns a typed status/diagnostic instead of failing read-oriented commands. Wire `prepare-task` and `get-pack` to use it by default, and add an additive `--no-trace` flag if needed. Preserve existing CLI command names and output formats; do not wrap existing array JSON outputs unless behind an explicit flag. Extend `cli_compat` with real temp repo tests for stale inventory rebuild diagnostics, read-only/constrained home trace failure, and help output for any new CLI flag.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm --test cli_compat -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `prepare-task`/`get-pack` exit successfully even when trace append cannot write.
    - JSON output exposes source-free diagnostics and keeps existing top-level fields.
    - `cargo run -p ctxhelm -- --help` shows any new flag without breaking command listing.
  </acceptance_criteria>
  <done>CLI users can see diagnostics and retrieve context even when local trace writes are constrained or disabled.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Expose diagnostics through MCP tools and resources</name>
  <files>crates/ctxhelm-mcp/src/tools.rs, crates/ctxhelm-mcp/src/resources.rs, crates/ctxhelm-mcp/src/schemas.rs, crates/ctxhelm-mcp/src/lib.rs</files>
  <read_first>
    - `crates/ctxhelm-mcp/src/tools.rs`
    - `crates/ctxhelm-mcp/src/resources.rs`
    - `crates/ctxhelm-mcp/src/schemas.rs`
    - `crates/ctxhelm-mcp/src/lib.rs`
  </read_first>
  <behavior>
    - DIAG-02: MCP `structuredContent` exposes diagnostics for `prepare_task`, `get_pack`, `search`, `related`, `related_tests`, and `current_diff` where diagnostics exist.
    - SAFE-04: MCP file resources revalidate source-bearing paths through fresh safe inventory immediately before reading.
    - SAFE-06: MCP `prepare_task` and `get_pack` trace write failures are non-fatal and visible; optional trace control is additive if implemented.
    - DIAG-04: MCP tests cover weak plan diagnostics, stale cache diagnostics, file-resource exclusion, and trace-write failure.
  </behavior>
  <action>
    Wire compiler/index diagnostics into MCP tool `structuredContent` while preserving text fallback and the six-tool surface. Replace MCP file resource direct reads with the same `load_or_refresh_inventory` plus `read_safe_source` path used by packs. Convert trace append calls to the non-fatal trace status helper. Add tests in `lib.rs` for `structuredContent.diagnostics`, stale-cache reporting, file resources rejecting a path that became sensitive/generated/deleted, and trace-write failures not becoming JSON-RPC errors.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-mcp diagnostics -- --nocapture</automated>
    <automated>cargo test -p ctxhelm-mcp file_resource -- --nocapture</automated>
    <automated>cargo test -p ctxhelm-mcp trace -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - MCP `tools/list` still exposes exactly the existing six tools.
    - MCP tool responses include diagnostics in machine-readable structured content without removing existing fields.
    - MCP file-resource reads never return source for paths that fail current safe policy.
  </acceptance_criteria>
  <done>MCP clients receive explicit diagnostics and safe file-resource behavior without tool-surface expansion.</done>
</task>

<task type="auto">
  <name>Task 3: Run full Phase 2 validation</name>
  <files>crates/ctxhelm-core/src/contracts.rs, crates/ctxhelm-index/src/*.rs, crates/ctxhelm-compiler/src/*.rs, crates/ctxhelm-mcp/src/*.rs, crates/ctxhelm/src/main.rs, crates/ctxhelm/tests/*.rs</files>
  <read_first>
    - `AGENTS.md`
    - `.planning/phases/02-trust-layer-operational-diagnostics/02-RESEARCH.md`
    - `.planning/phases/01-compatibility-guardrails-module-boundaries/01-VERIFICATION.md`
  </read_first>
  <behavior>
    - All Phase 2 requirements listed in this plan are covered by automated tests or public contract checks.
    - Phase 1 compatibility guardrails still pass after diagnostics/freshness/wiring changes.
    - No Phase 3 ranking, signal-ablation, parser-upgrade, or historical-eval failure-grouping work is introduced.
  </behavior>
  <action>
    Run the focused Phase 2 crate tests, full workspace tests, and CLI help validation. If validation fails, fix only failures caused by Phase 2 changes. Do not broaden scope into retrieval lift or eval-gate work.
  </action>
  <verify>
    <automated>cargo test --workspace</automated>
    <automated>cargo run -p ctxhelm -- --help</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test --workspace` passes.
    - `cargo run -p ctxhelm -- --help` passes.
    - CLI and MCP compatibility guardrails still pass.
  </acceptance_criteria>
  <done>Phase 2 implementation is validated end-to-end against workspace tests and CLI help.</done>
</task>

</tasks>

<verification>
```bash
cargo test -p ctxhelm --test cli_compat -- --nocapture
cargo test -p ctxhelm-mcp diagnostics -- --nocapture
cargo test -p ctxhelm-mcp file_resource -- --nocapture
cargo test -p ctxhelm-mcp trace -- --nocapture
cargo test --workspace
cargo run -p ctxhelm -- --help
```
</verification>

<success_criteria>
SAFE-06 and DIAG-02 are visible at the CLI/MCP boundaries, SAFE-04 file-resource safety is wired, and the full Phase 2 trust layer passes workspace validation without Phase 3 retrieval-ranking work.
</success_criteria>

<output>
After completion, create `.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-05-SUMMARY.md`.
</output>
