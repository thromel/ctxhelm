---
phase: 06-agent-setup-first-pack-adoption
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/ctxhelm-core/src/init.rs
  - crates/ctxhelm/src/main.rs
  - crates/ctxhelm/tests/cli_compat.rs
autonomous: true
requirements: [ADPT-01]
must_haves:
  truths:
    - "User can run `ctxhelm init` and see which repo-local files were created, updated, unchanged, or skipped."
    - "User can rerun `ctxhelm init` and see unchanged files rather than guessing whether setup was idempotent."
    - "User can see exact next actions after init: verify the binary, validate setup artifacts, prove MCP, configure an agent explicitly, then request a first context plan/pack."
  artifacts:
    - path: "crates/ctxhelm-core/src/init.rs"
      provides: "Structured init report actions and next-step contract"
    - path: "crates/ctxhelm/src/main.rs"
      provides: "Human-readable init report rendering"
    - path: "crates/ctxhelm/tests/cli_compat.rs"
      provides: "Binary-level init report compatibility tests"
  key_links:
    - from: "crates/ctxhelm/src/main.rs"
      to: "ctxhelm_core::run_init"
      via: "Command::Init dispatch renders InitReport"
      pattern: "print_init_report"
    - from: "crates/ctxhelm-core/src/init.rs"
      to: "generated adapter files"
      via: "InitAction records created/updated/unchanged/skipped per path"
      pattern: "InitAction::Skipped|skipped"
---

<objective>
Make `ctxhelm init` an actionable first-run report.

Purpose: Phase 6 adoption starts with a user understanding exactly what init changed, what it deliberately did not change, and what to do next.
Output: Structured init report state plus CLI output that names repo-local file actions and next setup/smoke steps.
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
@.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-04-SUMMARY.md
@crates/ctxhelm-core/src/init.rs
@crates/ctxhelm/src/main.rs
@crates/ctxhelm/tests/cli_compat.rs

<decision_trace>
- ADPT-01: `ctxhelm init` must report repo-local files written, skipped, or left unchanged and provide exact next setup/smoke steps.
- Phase 6 context: init must keep repo-local writes only and must not mutate global agent configuration.
- Phase 5 dependency: binary users already have `ctxhelm --version` and `ctxhelm --help` as release diagnostics.
</decision_trace>

<interfaces>
From `crates/ctxhelm-core/src/init.rs`:
```rust
pub enum AgentAdapter { Cursor, Claude, OpenCode }
pub struct InitOptions { pub adapters: Vec<AgentAdapter> }
pub enum InitAction { Created, Updated, Unchanged }
pub struct InitFile { pub path: PathBuf, pub action: InitAction }
pub struct InitReport {
    pub repo_root: PathBuf,
    pub files: Vec<InitFile>,
    pub codex_mcp_setup: String,
}
pub fn run_init(repo_root: impl AsRef<Path>, options: &InitOptions) -> Result<InitReport, InitError>;
pub fn adapter_files(adapter: AgentAdapter) -> Vec<(&'static str, &'static str)>;
```

From `crates/ctxhelm/src/main.rs`:
```rust
Command::Init(args) => {
    let report = run_init(&repo.path, &init_options(&args))?;
    print_init_report(&report);
}
fn print_init_report(report: &InitReport);
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add structured init actions and next steps</name>
  <files>crates/ctxhelm-core/src/init.rs</files>
  <behavior>
    - Test 1: `run_init(temp_repo, InitOptions::default())` reports `.ctxhelm/ctxhelm.toml` and `AGENTS.md` as `created`, and reports optional Cursor, Claude, and OpenCode adapter paths as `skipped` because those flags were not requested.
    - Test 2: Rerunning `run_init` with the same options reports previously written files as `unchanged` and the same unrequested adapter paths as `skipped`.
    - Test 3: `run_init` with all adapters requested reports `.cursor/rules/ctxhelm.mdc`, `.claude/commands/ctxhelm-bugfix.md`, `.ctxhelm/adapters/claude-mcp.json`, and `.ctxhelm/adapters/opencode.jsonc.snippet` as created or unchanged, not skipped.
    - Test 4: Serialized `InitReport` keeps camelCase keys and includes a stable next-step list with commands containing `ctxhelm --version`, `ctxhelm --help`, `ctxhelm setup-check --repo`, `prepare_task`, and `get_pack`.
  </behavior>
  <action>Extend the init report contract per ADPT-01. Add `Skipped` to `InitAction` and record skipped entries for optional adapter files that were not requested. Add a typed `InitNextStep` or equivalent field to `InitReport` with concise label/command/detail values for the post-init ladder: verify binary, validate generated setup, prove MCP, configure one agent explicitly, and request first context. Keep changes additive and serde-compatible where possible: existing fields remain, field names stay camelCase, and no global agent config write is introduced.</action>
  <verify>
    <automated>cargo test -p ctxhelm-core init_report -- --nocapture && cargo test -p ctxhelm-core init_creates_config_agents_and_requested_adapters -- --nocapture</automated>
  </verify>
  <done>`run_init` returns enough structured information to distinguish created, updated, unchanged, and skipped files and to drive a first-run next-step report.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Render actionable init output in the CLI</name>
  <files>crates/ctxhelm/src/main.rs, crates/ctxhelm/tests/cli_compat.rs</files>
  <behavior>
    - Test 1: `ctxhelm init --repo <repo>` prints the initialized repo root, grouped or clearly labeled file actions including `created`, `unchanged`, and `skipped`.
    - Test 2: `ctxhelm init --repo <repo> --cursor --claude --opencode` prints all generated adapter file paths and does not claim to mutate global Codex, Claude, Cursor, or OpenCode config.
    - Test 3: The init output includes a short `Next steps` section with exact commands or MCP tool names for binary verification, setup validation, deterministic MCP proof, explicit agent setup, `prepare_task`, and `get_pack`.
    - Test 4: A second init run prints `unchanged` for files already matching generated content.
  </behavior>
  <action>Update `print_init_report` to render the new structured report. Keep the output plain text and compact: show repo root, file action lines, and a `Next steps` block. For skipped adapters, explain that the user can rerun init with `--cursor`, `--claude`, or `--opencode`. Preserve existing Codex guidance, but make it clear it is copy/paste guidance only. Add binary-level tests in `crates/ctxhelm/tests/cli_compat.rs` using temp repos and `assert_cmd`.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test cli_compat init -- --nocapture && tmp="$(mktemp -d)" && repo="$tmp/repo" && mkdir "$repo" && git -C "$repo" init >/dev/null && cargo run -p ctxhelm -- init --repo "$repo" --cursor --claude --opencode</automated>
  </verify>
  <done>`ctxhelm init` gives a first-time user a clear setup report and exact next actions without modifying global agent state.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm-core init -- --nocapture`
- `cargo test -p ctxhelm --test cli_compat init -- --nocapture`
- `tmp="$(mktemp -d)" && repo="$tmp/repo" && mkdir "$repo" && git -C "$repo" init >/dev/null && cargo run -p ctxhelm -- init --repo "$repo" --cursor --claude --opencode`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
</verification>

<success_criteria>
Plan 01 is complete when ADPT-01 is satisfied by a structured and human-readable init report that identifies created, updated, unchanged, and skipped repo-local setup artifacts and tells the user what to do next.
</success_criteria>

<output>
After completion, create `.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-01-SUMMARY.md`
</output>
