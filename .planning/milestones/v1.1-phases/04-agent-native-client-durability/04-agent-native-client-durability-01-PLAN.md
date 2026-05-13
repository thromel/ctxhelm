---
phase: 04-agent-native-client-durability
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/ctxpack/tests/cli_compat.rs
  - scripts/smoke-mcp-protocol.sh
autonomous: true
requirements: [AGNT-01, AGNT-03]
must_haves:
  truths:
    - "Maintainer can run one deterministic MCP stdio smoke that calls initialize, prepare_task, get_pack, and a pack resource read."
    - "The MCP server can be launched from a directory outside the target repo while all repo-accepting MCP tools use the explicit repo argument."
    - "Smoke validation fails if prepare_task, get_pack, search, related, related_tests, or current_diff returns empty or wrong-repo structured content."
  artifacts:
    - path: "scripts/smoke-mcp-protocol.sh"
      provides: "Hard-gate protocol smoke for stdio MCP prepare_task/get_pack with explicit repo"
    - path: "crates/ctxpack/tests/cli_compat.rs"
      provides: "Binary-level regression test for wrong-cwd explicit-repo stdio behavior"
  key_links:
    - from: "scripts/smoke-mcp-protocol.sh"
      to: "ctxpack serve-mcp"
      via: "JSON-RPC stdio launched with --manifest-path from outside the repo"
      pattern: "serve-mcp"
    - from: "crates/ctxpack/tests/cli_compat.rs"
      to: "crates/ctxpack-mcp/src/tools.rs"
      via: "compiled binary tools/call prepare_task and get_pack requests"
      pattern: "\"repo\""
---

<objective>
Create the deterministic MCP protocol smoke that Phase 4 treats as the hard gate.

Purpose: Real client smokes can be flaky, but ctxpack still needs a repeatable proof that the MCP transport calls the right dynamic tools for the right repo.
Output: A scriptable stdio MCP smoke plus a binary compatibility test for explicit-repo wrong-cwd behavior.
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
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-05-SUMMARY.md
@crates/ctxpack/tests/cli_compat.rs
@crates/ctxpack/tests/common/mod.rs
@crates/ctxpack-mcp/src/protocol.rs
@crates/ctxpack-mcp/src/tools.rs

<decision_trace>
- D-01: Deterministic MCP stdio protocol smoke is the hard gate for this phase.
- D-02: MCP tool smokes must pass explicit `repo` and prove wrong-cwd behavior does not silently use the server cwd.
- D-05: Do not add cloud/vector/editor-app scope or widen the MCP tool list for this task.
</decision_trace>

<interfaces>
From `crates/ctxpack-mcp/src/protocol.rs`:
```rust
pub fn run_stdio_server() -> io::Result<()>;
pub fn run_server<R, W>(reader: R, writer: W) -> io::Result<()>
where
    R: BufRead,
    W: Write;
```

From `crates/ctxpack-mcp/src/tools.rs`:
```rust
struct PrepareTaskArgs {
    task: String,
    repo: Option<PathBuf>,
    mode: Option<TaskType>,
    paths: Vec<String>,
    include_current_diff: bool,
    target_agent: Option<String>,
    record_trace: bool,
}

struct GetPackArgs {
    task: String,
    repo: Option<PathBuf>,
    mode: Option<TaskType>,
    budget: Option<PackBudget>,
    format: Option<PackFormat>,
    paths: Vec<String>,
    include_current_diff: bool,
    target_agent: Option<String>,
    record_trace: bool,
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add deterministic wrong-cwd protocol smoke coverage</name>
  <files>crates/ctxpack/tests/cli_compat.rs, scripts/smoke-mcp-protocol.sh</files>
  <behavior>
    - Test 1: A binary test starts `ctxpack serve-mcp` from a temp directory outside the fixture repo, sends `prepare_task` with `repo`, and sees `src/auth/session.ts` in structuredContent.
    - Test 2: The same binary test sends `get_pack` with the same explicit `repo`, `budget=brief`, `format=json`, `recordTrace=false`, and sees a non-empty repoId plus pack sections.
    - Test 3: The same binary test sends `search`, `related`, `related_tests`, and `current_diff` with explicit `repo` from the wrong cwd and verifies each response is successful and points at fixture data rather than server-cwd data.
    - Test 4: `scripts/smoke-mcp-protocol.sh` runs from the project root, launches `cargo run --manifest-path "$CTXPACK_ROOT/Cargo.toml" -p ctxpack -- serve-mcp` with server cwd outside the target repo, sends initialize/tools/call requests for prepare_task, get_pack, search, related, related_tests, current_diff, and validates JSON with `python3`.
    - Test 5: The smoke script exits non-zero if prepare_task/get_pack omit structuredContent, targetFiles/sections are empty, repoId is blank, any repo-accepting tool fails from the wrong cwd, or the explicit repo path is not accepted.
  </behavior>
  <action>Implement the protocol hard gate per D-01 and D-02. Add a focused compatibility test in `crates/ctxpack/tests/cli_compat.rs` using the existing fixture helpers and `assert_cmd::Command::cargo_bin("ctxpack")`; set `current_dir` to a temp directory that is not the fixture repo, pass `repo` explicitly in all repo-accepting MCP tool calls, and parse each JSON-RPC response line. Create `scripts/smoke-mcp-protocol.sh` as a portable bash script with `set -euo pipefail`, defaults `CTXPACK_ROOT=$PWD`, `CTXPACK_SMOKE_REPO=$PWD`, `CTXPACK_SMOKE_TASK="fix requireSession auth bug"`, and `CTXPACK_HOME=$(mktemp -d)`. The script must build/send newline-delimited JSON-RPC for initialize, `tools/call prepare_task`, `tools/call get_pack`, `tools/call search`, `tools/call related`, `tools/call related_tests`, `tools/call current_diff`, and a `resources/read` for the first returned pack resource URI from the same server process. Keep validation structured with `python3`; do not scrape prose, do not depend on a real client, and do not add MCP tools.</action>
  <verify>
    <automated>cargo test -p ctxpack --test cli_compat mcp_protocol -- --nocapture && CTXPACK_SMOKE_REPO="$PWD" CTXPACK_SMOKE_TASK="harden MCP explicit repo handling" bash scripts/smoke-mcp-protocol.sh</automated>
  </verify>
  <done>The hard-gate protocol smoke and binary test prove prepare_task/get_pack work through stdio from the wrong cwd when `repo` is explicit.</done>
</task>

<task type="auto">
  <name>Task 2: Run the protocol gate with workspace validation</name>
  <files>None</files>
  <action>Run the protocol gate after Task 1 and then the standard project validation. If the deterministic protocol smoke fails, fix the owning test/script or MCP handler before proceeding to real-client plans. Do not treat Codex CLI or Claude Code smoke success as a substitute for this protocol gate.</action>
  <verify>
    <automated>CTXPACK_SMOKE_REPO="$PWD" CTXPACK_SMOKE_TASK="harden MCP explicit repo handling" bash scripts/smoke-mcp-protocol.sh && cargo test --workspace && cargo run -p ctxpack -- --help</automated>
  </verify>
  <done>Phase 4 has a repeatable deterministic MCP smoke that passes together with workspace tests and CLI help.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack --test cli_compat mcp_protocol -- --nocapture`
- `CTXPACK_SMOKE_REPO="$PWD" CTXPACK_SMOKE_TASK="harden MCP explicit repo handling" bash scripts/smoke-mcp-protocol.sh`
- `cargo test --workspace`
- `cargo run -p ctxpack -- --help`
</verification>

<success_criteria>
Plan 01 is complete when protocol-level MCP prepare_task/get_pack and same-session pack resource reads are deterministic, explicit-repo, wrong-cwd safe, and runnable without Codex CLI or Claude Code.
</success_criteria>

<output>
After completion, create `.planning/phases/04-agent-native-client-durability/04-agent-native-client-durability-01-SUMMARY.md`
</output>
