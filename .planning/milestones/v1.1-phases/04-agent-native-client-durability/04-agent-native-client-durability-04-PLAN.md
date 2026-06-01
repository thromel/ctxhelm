---
phase: 04-agent-native-client-durability
plan: 04
type: execute
wave: 3
depends_on: [01, 03]
files_modified:
  - scripts/smoke-codex-mcp.sh
  - scripts/smoke-claude-mcp.sh
  - crates/ctxhelm/tests/cli_compat.rs
autonomous: true
requirements: [AGNT-01]
must_haves:
  truths:
    - "Codex CLI smoke is available as an optional wrapper that first runs the deterministic protocol gate and then attempts a real MCP prepare_task/get_pack path when `codex` is installed."
    - "Claude Code smoke is available as an optional wrapper that first runs the deterministic protocol gate and then attempts a real MCP prepare_task/get_pack path when `claude` is installed."
    - "Both real-client wrappers use explicit repo arguments, isolate temporary client config/state, and report skipped, passed, or failed status clearly."
    - "When installed/authenticated clients are available, required mode attempts real-client invocation and requires machine-checkable tool-call evidence."
    - "A required mode exists so maintainers can fail CI/local validation if an installed real client cannot exercise ctxhelm."
  artifacts:
    - path: "scripts/smoke-codex-mcp.sh"
      provides: "Optional Codex CLI real-client MCP smoke"
    - path: "scripts/smoke-claude-mcp.sh"
      provides: "Optional Claude Code real-client MCP smoke"
    - path: "crates/ctxhelm/tests/cli_compat.rs"
      provides: "Script existence/syntax/contract guard for real-client smoke wrappers"
  key_links:
    - from: "scripts/smoke-codex-mcp.sh"
      to: "scripts/smoke-mcp-protocol.sh"
      via: "protocol hard gate before optional Codex exec"
      pattern: "smoke-mcp-protocol.sh"
    - from: "scripts/smoke-claude-mcp.sh"
      to: "scripts/smoke-mcp-protocol.sh"
      via: "protocol hard gate before optional claude --print"
      pattern: "smoke-mcp-protocol.sh"
---

<objective>
Add optional real-client Codex CLI and Claude Code smokes on top of the deterministic protocol gate.

Purpose: Users care about real coding-agent clients, but their model-driven tool invocation can be version/auth dependent, so the scripts must be useful evidence without replacing the deterministic hard gate.
Output: Scriptable Codex and Claude wrappers with explicit repo arguments, isolated config, clear skip/fail reporting, and compatibility guards.
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
@scripts/smoke-mcp-protocol.sh
@crates/ctxhelm/tests/cli_compat.rs

<decision_trace>
- D-01: Codex CLI and Claude Code smokes are optional/well-reported; deterministic MCP protocol smoke remains the hard gate.
- D-02: Real-client smokes must exercise prepare_task/get_pack with explicit `repo` when available.
- D-05: Do not mutate global user client config or add cloud/vector/editor-app scope.
</decision_trace>

<interfaces>
Local CLI help verified on 2026-05-13:
```text
codex exec [OPTIONS] [PROMPT]
  --cd <DIR>
  --skip-git-repo-check
  --ephemeral
  --ignore-user-config
  --json
  -o, --output-last-message <FILE>
  -c, --config <key=value>

claude -p/--print [options] [prompt]
  --mcp-config <configs...>
  --strict-mcp-config
  --allowedTools <tools...>
  --permission-mode <mode>
  --output-format <format>
  --no-session-persistence
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add script contract tests for real-client wrappers</name>
  <files>crates/ctxhelm/tests/cli_compat.rs, scripts/smoke-codex-mcp.sh, scripts/smoke-claude-mcp.sh</files>
  <behavior>
    - Test 1: Both scripts exist, are executable or runnable with `bash`, and pass `bash -n`.
    - Test 2: Both scripts invoke `scripts/smoke-mcp-protocol.sh` before any real-client attempt.
    - Test 3: Both scripts contain explicit `prepare_task`, `get_pack`, and `repo` instructions for the client prompt/config path.
    - Test 4: Both scripts support a required mode variable such as `CTXHELM_REQUIRE_REAL_CLIENT=1` and a default optional mode that can skip unavailable clients with a zero exit.
  </behavior>
  <action>Add compatibility tests in `crates/ctxhelm/tests/cli_compat.rs` that read the script files and run `bash -n` on them. Create initial script files if needed so the tests can go green in Task 2. The tests must guard contract text and skip/required semantics, not depend on Codex or Claude authentication.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test cli_compat real_client_smoke_scripts -- --nocapture</automated>
  </verify>
  <done>Real-client smoke scripts have syntax and contract guards independent of installed/authenticated client state.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement Codex and Claude real-client smoke wrappers</name>
  <files>scripts/smoke-codex-mcp.sh, scripts/smoke-claude-mcp.sh</files>
  <behavior>
    - Test 1: Each wrapper always runs `scripts/smoke-mcp-protocol.sh` first using `CTXHELM_SMOKE_REPO`, `CTXHELM_SMOKE_TASK`, and an isolated `CTXHELM_HOME`.
    - Test 2: If the client binary is absent and `CTXHELM_REQUIRE_REAL_CLIENT` is unset, the wrapper prints a clear skip message and exits 0.
    - Test 3: If the client binary is absent or cannot produce contract evidence and `CTXHELM_REQUIRE_REAL_CLIENT=1`, the wrapper exits non-zero.
    - Test 4: When the client path runs, the prompt/config explicitly asks for MCP `prepare_task` and `get_pack` calls with the absolute `repo`, then validates machine-checkable tool-call evidence rather than prose-only markers: Codex `--json` event/tool transcript, Claude structured output/tool transcript, or server-side smoke instrumentation proving both tools were called with the explicit repo.
    - Test 5: Default verification attempts installed clients unless `CTXHELM_SKIP_REAL_CLIENT=1`; required mode with `CTXHELM_REQUIRE_REAL_CLIENT=1` fails if an installed/authenticated client cannot produce the tool-call evidence.
  </behavior>
  <action>Implement `scripts/smoke-codex-mcp.sh` and `scripts/smoke-claude-mcp.sh` per D-01 and D-02. For Codex, isolate `CODEX_HOME` in a temp dir, use `codex exec --ephemeral --ignore-user-config --skip-git-repo-check --cd "$outside_cwd" --json --output-last-message "$out"` with `-c` MCP server config overrides or a temp config that launches the local ctxhelm binary/manifest path. For Claude, create a temp MCP config JSON and run `claude -p --bare --no-session-persistence --strict-mcp-config --mcp-config "$config" --allowedTools "mcp__ctxhelm__prepare_task,mcp__ctxhelm__get_pack" --permission-mode bypassPermissions --output-format stream-json` when available. Both prompts must require explicit `repo="$CTXHELM_SMOKE_REPO"` in prepare_task/get_pack. The scripts must validate machine-checkable tool-call evidence from JSON event/tool transcripts or explicit server-side instrumentation that records tool name plus repo path for both `prepare_task` and `get_pack`; a final assistant marker or last-message text alone is not sufficient. Treat auth/model/client refusal as a clear optional skip unless `CTXHELM_REQUIRE_REAL_CLIENT=1`. Default verification should attempt installed clients; `CTXHELM_SKIP_REAL_CLIENT=1` is allowed only for syntax/contract tests and non-authenticated environments. Do not permanently edit user Codex/Claude config.</action>
  <verify>
    <automated>bash -n scripts/smoke-codex-mcp.sh && bash -n scripts/smoke-claude-mcp.sh && CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-codex-mcp.sh && CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-claude-mcp.sh && cargo test -p ctxhelm --test cli_compat real_client_smoke_scripts -- --nocapture</automated>
  </verify>
  <done>Codex and Claude smoke wrappers are runnable, optional by default, required-mode capable, explicit-repo, and built on the deterministic protocol hard gate.</done>
</task>

</tasks>

<verification>
- `bash -n scripts/smoke-codex-mcp.sh`
- `bash -n scripts/smoke-claude-mcp.sh`
- `CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-codex-mcp.sh`
- `CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/smoke-claude-mcp.sh`
- `CTXHELM_SMOKE_REPO="$PWD" bash scripts/smoke-mcp-protocol.sh`
- Default installed-client attempt: `CTXHELM_SMOKE_REPO="$PWD" bash scripts/smoke-codex-mcp.sh`
- Default installed-client attempt: `CTXHELM_SMOKE_REPO="$PWD" bash scripts/smoke-claude-mcp.sh`
- Required when installed/authenticated: `CTXHELM_REQUIRE_REAL_CLIENT=1 CTXHELM_SMOKE_REPO="$PWD" bash scripts/smoke-codex-mcp.sh`
- Required when installed/authenticated: `CTXHELM_REQUIRE_REAL_CLIENT=1 CTXHELM_SMOKE_REPO="$PWD" bash scripts/smoke-claude-mcp.sh`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
</verification>

<success_criteria>
Plan 04 is complete when Codex CLI and Claude Code have optional, well-reported real-client smoke wrappers that use explicit repo arguments and cannot be mistaken for the deterministic protocol hard gate.
</success_criteria>

<output>
After completion, create `.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-04-SUMMARY.md`
</output>
