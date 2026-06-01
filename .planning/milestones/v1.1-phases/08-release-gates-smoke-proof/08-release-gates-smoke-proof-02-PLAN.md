---
phase: 08-release-gates-smoke-proof
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - scripts/smoke-codex-mcp.sh
  - scripts/smoke-claude-mcp.sh
  - crates/ctxhelm/tests/cli_compat.rs
autonomous: true
requirements: [SMOKE-03]
must_haves:
  truths:
    - "Optional Codex CLI smoke records exact Codex and ctxhelm versions plus server-side prepare_task/get_pack evidence when it runs."
    - "Optional Claude Code smoke records exact Claude and ctxhelm versions plus server-side prepare_task/get_pack evidence when it runs."
    - "Real-client smokes use the selected `CTXHELM_BIN` path when provided and remain optional unless `CTXHELM_REQUIRE_REAL_CLIENT=1` is set."
    - "Skipped real-client checks clearly distinguish missing/unavailable clients from failed required evidence."
  artifacts:
    - path: "scripts/smoke-codex-mcp.sh"
      provides: "Optional Codex CLI real-client smoke with selected binary and versioned machine-checkable evidence"
    - path: "scripts/smoke-claude-mcp.sh"
      provides: "Optional Claude Code real-client smoke with selected binary and versioned machine-checkable evidence"
    - path: "crates/ctxhelm/tests/cli_compat.rs"
      provides: "Contract tests for optional real-client smoke wrappers"
  key_links:
    - from: "scripts/smoke-codex-mcp.sh"
      to: "ctxhelm serve-mcp"
      via: "server wrapper launched through CTXHELM_BIN when selected"
      pattern: "CTXHELM_BIN|serve-mcp"
    - from: "scripts/smoke-claude-mcp.sh"
      to: "ctxhelm serve-mcp"
      via: "server wrapper launched through CTXHELM_BIN when selected"
      pattern: "CTXHELM_BIN|serve-mcp"
    - from: "scripts/smoke-codex-mcp.sh"
      to: "request log evidence"
      via: "server-side JSON-RPC tools/call instrumentation"
      pattern: "prepare_task|get_pack|clientVersion|ctxhelmVersion"
    - from: "scripts/smoke-claude-mcp.sh"
      to: "request log evidence"
      via: "server-side JSON-RPC tools/call instrumentation"
      pattern: "prepare_task|get_pack|clientVersion|ctxhelmVersion"
---

<objective>
Harden optional real-client smoke evidence.

Purpose: Phase 8 can only claim real Codex/Claude client proof when wrappers record exact client versions and machine-checkable `prepare_task` plus `get_pack` evidence with explicit repo arguments.
Output: Updated optional Codex and Claude smoke wrappers plus contract tests.
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
@.planning/research/SUMMARY.md
@.planning/research/PITFALLS.md
@.planning/phases/08-release-gates-smoke-proof/08-CONTEXT.md
@.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-04-SUMMARY.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-04-SUMMARY.md
@.planning/phases/07-documentation-troubleshooting/07-documentation-troubleshooting-02-SUMMARY.md
@scripts/smoke-codex-mcp.sh
@scripts/smoke-claude-mcp.sh
@scripts/smoke-mcp-protocol.sh
@crates/ctxhelm/tests/cli_compat.rs

<decision_trace>
- SMOKE-03: Optional Codex CLI and Claude Code real-client smokes must record machine-checkable `prepare_task` and `get_pack` evidence with exact client versions when required.
- Phase 8 decision: Codex/Claude real-client smokes remain optional unless `CTXHELM_REQUIRE_REAL_CLIENT=1` is set.
- Phase 6 decision: Deterministic protocol proof remains the hard gate; real clients depend on auth/client state.
- Scope guard: Do not claim Cursor/OpenCode real-client tool-call proof and do not mutate global agent configuration.
</decision_trace>

<interfaces>
Current wrapper behavior to preserve and improve:

```text
scripts/smoke-codex-mcp.sh and scripts/smoke-claude-mcp.sh:
- run `scripts/smoke-mcp-protocol.sh` first
- support `CTXHELM_SKIP_REAL_CLIENT=1`
- support `CTXHELM_REQUIRE_REAL_CLIENT=1`
- use temp homes/configs and a request log
- inspect server-side JSON-RPC tools/call records for `prepare_task` and `get_pack` with explicit repo
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add real-client evidence contract tests</name>
  <files>crates/ctxhelm/tests/cli_compat.rs, scripts/smoke-codex-mcp.sh, scripts/smoke-claude-mcp.sh</files>
  <behavior>
    - Test 1: Both wrapper scripts pass `bash -n`.
    - Test 2: Both wrappers propagate `CTXHELM_BIN` into the deterministic protocol gate and their generated MCP server wrapper.
    - Test 3: Both wrappers capture exact client version output (`codex --version` or equivalent, `claude --version` or equivalent) and ctxhelm version output from the selected binary.
    - Test 4: Both wrappers write or print machine-checkable evidence containing client name, client version, ctxhelm version, repo, `prepare_task`, and `get_pack`.
    - Test 5: Existing `CTXHELM_SKIP_REAL_CLIENT` and `CTXHELM_REQUIRE_REAL_CLIENT` behavior remains visible in the scripts.
  </behavior>
  <action>Extend `crates/ctxhelm/tests/cli_compat.rs` real-client smoke contract coverage. The tests should inspect wrapper content for selected-binary use, version capture, evidence fields, skip/require env semantics, and server-side request-log validation. Keep tests deterministic and non-authenticated; do not invoke real Codex or Claude from the Rust test.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test cli_compat real_client -- --nocapture</automated>
  </verify>
  <done>Real-client smoke evidence requirements fail in tests before wrapper implementation changes.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Record versioned real-client smoke evidence</name>
  <files>scripts/smoke-codex-mcp.sh, scripts/smoke-claude-mcp.sh, crates/ctxhelm/tests/cli_compat.rs</files>
  <behavior>
    - Test 1: If `CTXHELM_BIN` is set, both wrappers validate it is executable and use it for protocol smoke and stdio `serve-mcp`; if unset, they retain the current development fallback.
    - Test 2: When a real client is available and runs, each wrapper records exact client version, ctxhelm version, repo, and booleans or fields proving `prepare_task` and `get_pack` calls from the request log.
    - Test 3: If `CTXHELM_REAL_CLIENT_EVIDENCE_DIR` is set, each wrapper writes a stable JSON evidence file there; otherwise it prints a concise evidence summary and keeps temp artifacts private.
    - Test 4: If real-client evidence is missing and `CTXHELM_REQUIRE_REAL_CLIENT=1`, the wrapper exits non-zero; if not required, it skips with a clear diagnostic after deterministic protocol proof.
  </behavior>
  <action>Update both real-client wrappers to resolve a selected ctxhelm binary consistently, run the deterministic protocol gate with that binary, launch the real-client MCP server wrapper through that same binary, and collect exact versions before the client invocation. After request-log validation succeeds, emit a source-free JSON evidence object with fields like `client`, `clientVersion`, `ctxhelmVersion`, `repo`, `prepareTask`, `getPack`, and `required`. Do not log source snippets, prompt text beyond the existing smoke instruction, auth tokens, or temp paths unless needed for diagnostics.</action>
  <verify>
    <automated>cargo build -p ctxhelm && cargo test -p ctxhelm --test cli_compat real_client -- --nocapture && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-codex-mcp.sh && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-claude-mcp.sh</automated>
  </verify>
  <done>Optional real-client wrappers satisfy SMOKE-03 evidence requirements when clients run and remain safely skippable otherwise.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm --test cli_compat real_client -- --nocapture`
- `cargo build -p ctxhelm`
- `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-codex-mcp.sh`
- `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-claude-mcp.sh`
- Optional on provisioned maintainer machines: `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_REQUIRE_REAL_CLIENT=1 bash scripts/smoke-codex-mcp.sh`
- Optional on provisioned maintainer machines: `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_REQUIRE_REAL_CLIENT=1 bash scripts/smoke-claude-mcp.sh`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
</verification>

<success_criteria>
Plan 02 is complete when SMOKE-03 is satisfied by optional Codex and Claude wrappers that prove real-client use through versioned, source-free, machine-checkable request evidence.
</success_criteria>

<output>
After completion, create `.planning/phases/08-release-gates-smoke-proof/08-release-gates-smoke-proof-02-SUMMARY.md`
</output>
