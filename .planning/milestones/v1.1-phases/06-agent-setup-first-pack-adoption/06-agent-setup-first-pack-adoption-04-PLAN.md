---
phase: 06-agent-setup-first-pack-adoption
plan: 04
type: execute
wave: 4
depends_on: [03]
files_modified:
  - scripts/smoke-mcp-protocol.sh
  - scripts/smoke-first-pack.sh
  - crates/ctxhelm/tests/cli_compat.rs
autonomous: true
requirements: [ADPT-05]
must_haves:
  truths:
    - "User can run a first-pack smoke against an installed or explicitly selected ctxhelm binary, not only `cargo run`."
    - "The first-pack flow creates a real temp git repo, runs init, validates setup, proves deterministic MCP, then calls `prepare-task` and `get-pack`."
    - "The flow uses explicit `repo` arguments and produces machine-checkable JSON evidence for prepare-task/get-pack success."
  artifacts:
    - path: "scripts/smoke-mcp-protocol.sh"
      provides: "Protocol smoke runnable through selected `CTXHELM_BIN` as well as cargo fallback"
    - path: "scripts/smoke-first-pack.sh"
      provides: "Install-to-init-to-MCP-to-first-pack adoption smoke"
    - path: "crates/ctxhelm/tests/cli_compat.rs"
      provides: "Script contract and execution guard tests"
  key_links:
    - from: "scripts/smoke-first-pack.sh"
      to: "ctxhelm init/setup-check/prepare-task/get-pack"
      via: "selected CTXHELM_BIN command invocations with explicit repo"
      pattern: "CTXHELM_BIN|prepare-task|get-pack|setup-check"
    - from: "scripts/smoke-mcp-protocol.sh"
      to: "ctxhelm serve-mcp"
      via: "JSON-RPC stdio launched through CTXHELM_BIN when provided"
      pattern: "serve-mcp"
---

<objective>
Add a first-pack adoption smoke.

Purpose: The phase succeeds only when a user can get from an installed ctxhelm binary to a first useful plan and pack on a real repository-shaped fixture.
Output: A deterministic smoke script and tests that prove the first-pack journey end to end without real-agent auth.
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
@.planning/research/FEATURES.md
@.planning/research/PITFALLS.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-CONTEXT.md
@.planning/phases/05-release-identity-binary-packaging/VERIFICATION.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-03-SUMMARY.md
@scripts/smoke-mcp-protocol.sh
@scripts/smoke-codex-mcp.sh
@scripts/smoke-claude-mcp.sh
@crates/ctxhelm/tests/cli_compat.rs

<decision_trace>
- ADPT-05: User must complete a first-pack quickstart from install to init to deterministic MCP proof to prepare-task/get-pack on a real repo.
- Phase 6 context: deterministic MCP protocol smoke is the hard proof before asking users to debug agent auth/model issues.
- Scope guard: This is not the Phase 8 release gate and does not claim Cursor/OpenCode real-client tool-call validation.
</decision_trace>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Let deterministic MCP smoke use a selected binary</name>
  <files>scripts/smoke-mcp-protocol.sh, crates/ctxhelm/tests/cli_compat.rs</files>
  <behavior>
    - Test 1: `scripts/smoke-mcp-protocol.sh` passes `bash -n`.
    - Test 2: The script honors `CTXHELM_BIN=/path/to/ctxhelm` by launching `$CTXHELM_BIN serve-mcp` instead of `cargo run`.
    - Test 3: When `CTXHELM_BIN` is unset, the script keeps the existing cargo-run development fallback.
    - Test 4: The script still launches the server from a wrong cwd, passes explicit `repo`, and validates `prepare_task`, `get_pack`, `search`, `related`, `related_tests`, `current_diff`, and same-session pack resource reads.
  </behavior>
  <action>Modify `scripts/smoke-mcp-protocol.sh` to support `CTXHELM_BIN`. If set, validate that it is executable and use it as the command prefix for `serve-mcp`; if unset, preserve the current `cargo run --manifest-path "$CTXHELM_ROOT/Cargo.toml" -p ctxhelm -- serve-mcp` fallback. Update `crates/ctxhelm/tests/cli_compat.rs` to guard both script syntax and the presence/order of the CTXHELM_BIN path. Do not broaden this into a full release gate; the purpose here is first-pack adoption proof.</action>
  <verify>
    <automated>cargo build -p ctxhelm && cargo test -p ctxhelm --test cli_compat mcp_protocol -- --nocapture && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_PATH="crates/ctxhelm-mcp/src/lib.rs" bash scripts/smoke-mcp-protocol.sh</automated>
  </verify>
  <done>The deterministic MCP proof can run through the same binary path users configure in agents.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add first-pack quickstart smoke script</name>
  <files>scripts/smoke-first-pack.sh, crates/ctxhelm/tests/cli_compat.rs</files>
  <behavior>
    - Test 1: `scripts/smoke-first-pack.sh` passes `bash -n` and requires or resolves a runnable ctxhelm binary through `CTXHELM_BIN` or `command -v ctxhelm`; it must not require a source checkout for normal use.
    - Test 2: The script creates a temp git repo with at least one source file, one related test file, and enough content for `prepare-task` and `get-pack` to return non-empty JSON.
    - Test 3: The script runs `$CTXHELM_BIN --version`, `$CTXHELM_BIN --help`, `$CTXHELM_BIN init --repo "$repo" --cursor --claude --opencode`, `$CTXHELM_BIN setup-check --repo "$repo" --cursor --claude --opencode`, deterministic MCP protocol smoke with explicit `repo`, `$CTXHELM_BIN prepare-task ... --repo "$repo" --path ...`, and `$CTXHELM_BIN get-pack ... --repo "$repo" --budget brief --format json`.
    - Test 4: The script validates JSON outputs with Python: prepare-task has non-empty `targetFiles` and `packOptions`, and get-pack has non-empty `repoId` and `sections`.
    - Test 5: The script prints a concise success line naming the repo and binary path, without logging source snippets.
  </behavior>
  <action>Create `scripts/smoke-first-pack.sh` as the user-facing adoption smoke for ADPT-05. Use `set -euo pipefail`, isolated `CTXHELM_HOME`, temp repo setup, and explicit `repo` in every ctxhelm/MCP call. Reuse `scripts/smoke-mcp-protocol.sh` instead of duplicating JSON-RPC logic. Add a `cli_compat` test that runs the script against `env!("CARGO_BIN_EXE_ctxhelm")` or another cargo-built binary path so the contract is exercised in CI/local tests. Keep real Codex/Claude client smokes out of this script; they remain optional and versioned.</action>
  <verify>
    <automated>cargo build -p ctxhelm && cargo test -p ctxhelm --test cli_compat first_pack -- --nocapture && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" bash scripts/smoke-first-pack.sh</automated>
  </verify>
  <done>A user or maintainer can prove the full first-pack adoption flow with one deterministic local command and no real-agent auth.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm --test cli_compat mcp_protocol -- --nocapture`
- `cargo test -p ctxhelm --test cli_compat first_pack -- --nocapture`
- `cargo build -p ctxhelm && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" bash scripts/smoke-first-pack.sh`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
</verification>

<success_criteria>
Plan 04 is complete when ADPT-05 is satisfied by a deterministic first-pack smoke that uses an installed/selected ctxhelm binary, initializes a real repo fixture, validates setup artifacts, proves MCP, and obtains both a plan and a brief pack with explicit repo arguments.
</success_criteria>

<output>
After completion, create `.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-04-SUMMARY.md`
</output>
