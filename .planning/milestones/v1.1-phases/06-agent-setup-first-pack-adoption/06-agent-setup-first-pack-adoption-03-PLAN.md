---
phase: 06-agent-setup-first-pack-adoption
plan: 03
type: execute
wave: 3
depends_on: [01, 02]
files_modified:
  - crates/ctxhelm-core/src/init.rs
  - crates/ctxhelm/src/main.rs
  - crates/ctxhelm/tests/cli_compat.rs
autonomous: true
requirements: [ADPT-04]
must_haves:
  truths:
    - "User can validate generated setup artifacts after init without opening every file manually."
    - "Validation checks expected files, JSON/syntax shape, command availability, absolute-binary-path guidance, and absence of large static context injection."
    - "Validation is read-only and does not apply or mutate global Codex, Claude, Cursor, or OpenCode configuration."
  artifacts:
    - path: "crates/ctxhelm-core/src/init.rs"
      provides: "Structured setup validation report"
    - path: "crates/ctxhelm/src/main.rs"
      provides: "`ctxhelm setup-check` CLI command and report renderer"
    - path: "crates/ctxhelm/tests/cli_compat.rs"
      provides: "Binary-level setup-check tests"
  key_links:
    - from: "crates/ctxhelm/src/main.rs"
      to: "ctxhelm_core::run_setup_check"
      via: "Command::SetupCheck dispatch"
      pattern: "SetupCheck"
    - from: "crates/ctxhelm-core/src/init.rs"
      to: "generated adapter constants"
      via: "setup validation compares files against expected generated artifacts"
      pattern: "setup.*check|adapter_files"
---

<objective>
Add a user-facing setup validation path.

Purpose: First-run adoption needs a quick way to distinguish "files generated correctly" from PATH/config/client issues before users debug their coding agent.
Output: A read-only `ctxhelm setup-check` command backed by structured validation results.
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
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-01-SUMMARY.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-02-SUMMARY.md
@crates/ctxhelm-core/src/init.rs
@crates/ctxhelm/src/main.rs
@crates/ctxhelm/tests/cli_compat.rs

<decision_trace>
- ADPT-04: User must be able to validate setup artifacts and snippets for syntax, expected files, command availability, absolute binary path guidance, and no large static context injection.
- Phase 6 context: the planner may choose a new command, script, or init extension; this plan chooses a read-only `ctxhelm setup-check` command because it is user-facing, testable, and reusable by first-pack smoke.
- Out of scope: no global agent config mutation and no real-client Cursor/OpenCode tool-call proof.
</decision_trace>

<interfaces>
The implementation should create a small core validation contract in `crates/ctxhelm-core/src/init.rs`, for example:
```rust
pub enum SetupCheckStatus { Pass, Warn, Fail }
pub struct SetupCheckItem {
    pub name: String,
    pub status: SetupCheckStatus,
    pub detail: String,
}
pub struct SetupCheckReport {
    pub repo_root: PathBuf,
    pub items: Vec<SetupCheckItem>,
    pub passed: bool,
}
pub fn run_setup_check(repo_root: impl AsRef<Path>, options: &InitOptions) -> Result<SetupCheckReport, InitError>;
```
Use the exact names only if they fit the existing code; keep the public shape typed and serde-serializable if it is exposed through CLI JSON later.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add core setup validation report</name>
  <files>crates/ctxhelm-core/src/init.rs</files>
  <behavior>
    - Test 1: After `run_init` with `--cursor --claude --opencode` equivalent options, `run_setup_check` passes expected-file checks for `AGENTS.md`, `.ctxhelm/ctxhelm.toml`, `.cursor/rules/ctxhelm.mdc`, `.claude/commands/ctxhelm-bugfix.md`, `.ctxhelm/adapters/claude-mcp.json`, and `.ctxhelm/adapters/opencode.jsonc.snippet`.
    - Test 2: Setup check fails or reports a failing item when an expected generated file is missing.
    - Test 3: Setup check validates JSON syntax for `.ctxhelm/adapters/claude-mcp.json` and `.ctxhelm/adapters/opencode.jsonc.snippet`.
    - Test 4: Setup check fails or warns if generated guidance contains forbidden large-static-context phrases or exceeds the byte limits from Plan 02.
    - Test 5: Setup check includes an item that reminds users to use an absolute `ctxhelm` binary path when an agent process cannot find `ctxhelm` on PATH.
  </behavior>
  <action>Implement a read-only setup validation helper in `crates/ctxhelm-core/src/init.rs`. It should inspect generated repo-local files and template contents; it must not run client commands, edit files, or mutate global config. Use existing `InitOptions` to decide which optional adapter files are expected. Treat missing optional files as skipped unless the corresponding adapter was requested. Keep validation source-free and concise: report file paths and status, not full file contents.</action>
  <verify>
    <automated>cargo test -p ctxhelm-core setup_check -- --nocapture</automated>
  </verify>
  <done>Core setup validation can prove generated artifacts are present, syntactically valid where applicable, thin, and aligned with absolute-path troubleshooting guidance.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Expose `ctxhelm setup-check` through the CLI</name>
  <files>crates/ctxhelm/src/main.rs, crates/ctxhelm/tests/cli_compat.rs</files>
  <behavior>
    - Test 1: `ctxhelm setup-check --repo <repo> --cursor --claude --opencode` exits 0 and prints a concise report after init generated those artifacts.
    - Test 2: `ctxhelm setup-check --repo <repo> --cursor` exits non-zero or prints a failing status when the Cursor rule file is expected but absent.
    - Test 3: CLI help lists `setup-check`, and `ctxhelm setup-check --help` documents that it is read-only and validates generated setup artifacts.
    - Test 4: The setup-check report includes command availability guidance for `ctxhelm --version` and absolute binary path troubleshooting, without attempting to write global agent configs.
  </behavior>
  <action>Add a `SetupCheck` subcommand to the Clap CLI. Reuse the same adapter flags as `init` so users can validate the setup profile they generated. Render status lines using `pass`, `warn`, and `fail`, and return a non-zero process exit when required checks fail. Add binary-level tests in `crates/ctxhelm/tests/cli_compat.rs`. Do not add a new external dependency and do not call real Codex, Claude, Cursor, or OpenCode clients from this command.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test cli_compat setup_check -- --nocapture && tmp="$(mktemp -d)" && repo="$tmp/repo" && mkdir "$repo" && git -C "$repo" init >/dev/null && cargo run -p ctxhelm -- init --repo "$repo" --cursor --claude --opencode && cargo run -p ctxhelm -- setup-check --repo "$repo" --cursor --claude --opencode && cargo run -p ctxhelm -- --help</automated>
  </verify>
  <done>Users have a read-only setup validation command that catches missing or malformed generated artifacts before agent-client debugging starts.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm-core setup_check -- --nocapture`
- `cargo test -p ctxhelm --test cli_compat setup_check -- --nocapture`
- `tmp="$(mktemp -d)" && repo="$tmp/repo" && mkdir "$repo" && git -C "$repo" init >/dev/null && cargo run -p ctxhelm -- init --repo "$repo" --cursor --claude --opencode && cargo run -p ctxhelm -- setup-check --repo "$repo" --cursor --claude --opencode`
- `cargo run -p ctxhelm -- --help`
- `cargo test --workspace`
</verification>

<success_criteria>
Plan 03 is complete when ADPT-04 is satisfied by a read-only setup validation path that verifies generated files/snippets, syntax, command/path guidance, and thin dynamic setup content.
</success_criteria>

<output>
After completion, create `.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-03-SUMMARY.md`
</output>
