---
phase: 01-compatibility-guardrails-module-boundaries
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/ctxpack/Cargo.toml
  - crates/ctxpack/tests/common/mod.rs
  - crates/ctxpack/tests/cli_compat.rs
autonomous: true
requirements: [CONT-01]
must_haves:
  truths:
    - "Maintainer can run binary-level CLI tests for core ctxpack commands."
    - "CLI tests exercise explicit --repo handling and command-local CTXPACK_HOME."
    - "CLI tests assert output shape and write side effects without brittle full-output snapshots."
  artifacts:
    - path: "crates/ctxpack/tests/cli_compat.rs"
      provides: "Binary-level compatibility tests for ctxpack commands"
      contains: "assert_cmd::Command::cargo_bin"
    - path: "crates/ctxpack/tests/common/mod.rs"
      provides: "Real temp repo and CTXPACK_HOME fixture helpers"
      contains: "FixtureRepo"
    - path: "crates/ctxpack/Cargo.toml"
      provides: "CLI test dev dependencies"
      contains: "assert_cmd"
  key_links:
    - from: "crates/ctxpack/tests/cli_compat.rs"
      to: "crates/ctxpack/src/main.rs"
      via: "compiled ctxpack binary"
      pattern: "cargo_bin\\(\"ctxpack\"\\)"
    - from: "crates/ctxpack/tests/common/mod.rs"
      to: "CTXPACK_HOME"
      via: "command-local environment setup"
      pattern: "CTXPACK_HOME"
---

<objective>
Add binary-level CLI compatibility guardrails before any module splitting.

Purpose: CONT-01 and decisions D-01, D-04, D-05, and D-06 require tests that exercise the compiled `ctxpack` executable, not only library helpers.
Output: A CLI integration-test target with fixture helpers, JSON shape assertions, repo path checks, and write side-effect checks.
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
@.planning/codebase/STRUCTURE.md
@.planning/codebase/TESTING.md
@.planning/codebase/CONVENTIONS.md
@AGENTS.md
@Cargo.toml
@crates/ctxpack/Cargo.toml
@crates/ctxpack/src/main.rs

<interfaces>
Current CLI command surface from `crates/ctxpack/src/main.rs` includes `init`, `index`, `prepare-task`, `get-pack`, `search`, `symbols`, `related-tests`, `co-changes`, `dependencies`, `cards generate`, `eval traces`, `eval checklist`, `eval history`, and `serve-mcp`.

Use `assert_cmd::Command::cargo_bin("ctxpack")` so tests execute the compiled binary. Use `serde_json::Value` for JSON shape checks and `predicates::str::contains` only for help or Markdown headings.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add CLI integration-test dependencies and fixture helpers</name>
  <files>crates/ctxpack/Cargo.toml, crates/ctxpack/tests/common/mod.rs</files>
  <read_first>
    - crates/ctxpack/Cargo.toml
    - crates/ctxpack/src/main.rs
    - crates/ctxpack-index/src/lib.rs
    - crates/ctxpack-compiler/src/lib.rs
    - .planning/codebase/TESTING.md
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
  </read_first>
  <behavior>
    - A fixture helper creates a real git repo with `src/auth/session.ts`, `tests/auth/session.test.ts`, `package.json`, `pnpm-lock.yaml`, `.env`, and `dist/generated.min.js`.
    - The helper creates an isolated home directory and exposes it so tests can set `CTXPACK_HOME`.
    - The helper runs `git init`, sets local user email/name, commits fixture files, and returns stable repo/home paths.
  </behavior>
  <action>
    Add dev-dependencies to `crates/ctxpack/Cargo.toml`: `assert_cmd = "2.2.2"` and `predicates = "3.1.4"` per D-06 and research. Create `crates/ctxpack/tests/common/mod.rs` with `pub struct FixtureRepo { pub temp: tempfile::TempDir, pub repo: PathBuf, pub home: PathBuf }`, `pub fn fixture_repo() -> FixtureRepo`, `pub fn run_git(repo: &Path, args: &[&str])`, and `pub fn json_stdout(assert: assert_cmd::assert::Assert) -> serde_json::Value`. Match the existing temp-repo style from `.planning/codebase/TESTING.md`; do not add `snapbox`, `trycmd`, `insta`, `schemars`, or source snapshots.
  </action>
  <verify>
    <automated>cargo test -p ctxpack --no-run</automated>
  </verify>
  <acceptance_criteria>
    - `rg -n 'assert_cmd = "2\\.2\\.2"|predicates = "3\\.1\\.4"' crates/ctxpack/Cargo.toml` finds both dev dependencies.
    - `test -f crates/ctxpack/tests/common/mod.rs` succeeds.
    - `rg -n 'pub struct FixtureRepo|pub fn fixture_repo|CTXPACK_HOME|git|src/auth/session\\.ts|tests/auth/session\\.test\\.ts' crates/ctxpack/tests/common/mod.rs` finds all listed strings.
  </acceptance_criteria>
  <done>CLI integration-test support exists and compiles without running source behavior changes.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Cover core CLI commands through the compiled binary</name>
  <files>crates/ctxpack/tests/cli_compat.rs</files>
  <read_first>
    - crates/ctxpack/tests/cli_compat.rs
    - crates/ctxpack/tests/common/mod.rs
    - crates/ctxpack/src/main.rs
    - crates/ctxpack-core/src/contracts.rs
    - crates/ctxpack-compiler/src/lib.rs
    - crates/ctxpack-index/src/lib.rs
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-RESEARCH.md
  </read_first>
  <behavior>
    - `ctxpack --help` exits 0 and mentions `index`, `prepare-task`, `get-pack`, `search`, `related-tests`, `dependencies`, `eval`, and `serve-mcp`.
    - `ctxpack index --repo <repo>` writes inventory under the temp `CTXPACK_HOME` and excludes `.env` and `dist/generated.min.js`.
    - `prepare-task`, `get-pack --format json`, `search`, `related-tests`, `dependencies`, and `eval history --format json --limit 1` return parseable JSON with current public camelCase fields.
    - `get-pack --format markdown` contains `# Context Pack`, `Repo ID:`, `Task Hash:`, and `Target Agent:` headings/fields.
    - `serve-mcp` accepts newline-delimited `initialize` and `tools/list` requests and returns one JSON-RPC response per request line.
  </behavior>
  <action>
    Create `crates/ctxpack/tests/cli_compat.rs`. Add tests named exactly `help_lists_core_commands`, `index_writes_inventory_under_command_home`, `prepare_task_outputs_context_plan_shape`, `get_pack_outputs_json_and_markdown_contracts`, `search_related_tests_dependencies_and_eval_history_emit_json_shapes`, and `serve_mcp_speaks_json_rpc_over_stdio`. Use `Command::cargo_bin("ctxpack")`, `.env("CTXPACK_HOME", &fixture.home)`, and `.args([...])` for every binary call. Parse JSON with `serde_json::from_slice::<Value>` and assert stable keys: `taskId`, `taskType`, `targetFiles`, `relatedTests`, `recommendedCommands`, `packOptions`, `riskFlags`, `missingInfoQuestions`, `confidence`, `privacyStatus`, `repoId`, `taskHash`, `targetAgent`, `budget`, `sections`, `sourceTextLogged`, `sourcePath`, `targetPath`, and `kind` where those fields apply. Keep assertions structured and substring-based; do not snapshot absolute paths or full dynamic output.
  </action>
  <verify>
    <automated>cargo test -p ctxpack --test cli_compat</automated>
    <automated>cargo run -p ctxpack -- --help</automated>
  </verify>
  <acceptance_criteria>
    - `rg -n 'help_lists_core_commands|index_writes_inventory_under_command_home|prepare_task_outputs_context_plan_shape|get_pack_outputs_json_and_markdown_contracts|search_related_tests_dependencies_and_eval_history_emit_json_shapes|serve_mcp_speaks_json_rpc_over_stdio' crates/ctxpack/tests/cli_compat.rs` finds all six test names.
    - `rg -n 'cargo_bin\\("ctxpack"\\)|CTXPACK_HOME|serde_json::from_slice|taskId|targetFiles|repoId|taskHash|targetAgent|sourceTextLogged|tools/list' crates/ctxpack/tests/cli_compat.rs` finds all compatibility markers.
    - `cargo test -p ctxpack --test cli_compat` passes.
    - `cargo run -p ctxpack -- --help` exits 0.
  </acceptance_criteria>
  <done>Binary-level tests cover the minimum command set from D-04 and prove repo path, output shape, and local write behavior.</done>
</task>

</tasks>

<verification>
Run `cargo test -p ctxpack --test cli_compat`, then `cargo run -p ctxpack -- --help`.
</verification>

<success_criteria>
CONT-01 is satisfied when a maintainer can run one Cargo integration-test target that executes the compiled `ctxpack` binary against real temp repositories and validates public CLI behavior without source snapshots.
</success_criteria>

<output>
After completion, create `.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-01-SUMMARY.md`.
</output>
