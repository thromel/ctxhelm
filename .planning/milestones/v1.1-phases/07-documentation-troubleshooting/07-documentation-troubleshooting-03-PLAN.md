---
phase: 07-documentation-troubleshooting
plan: 03
type: execute
wave: 2
depends_on: [01, 02]
files_modified:
  - README.md
  - docs/troubleshooting.md
  - scripts/check-release-docs.sh
  - crates/ctxpack/tests/release_packaging.rs
autonomous: true
requirements: [DOCS-02, DOCS-04]
must_haves:
  truths:
    - "User can troubleshoot PATH failures, absolute MCP binary paths, CTXPACK_HOME/state cleanup, wrong-cwd behavior, and common MCP startup failures from docs."
    - "User understands that `setup-check` validates repo-local artifacts only and does not run or mutate real agent clients."
    - "Docs consistently distinguish deterministic protocol proof from optional real-client proof and reject unsupported Cursor/OpenCode proof claims."
    - "A docs consistency gate checks README, quickstart, agent setup, release, and troubleshooting docs before Phase 8 release gates."
  artifacts:
    - path: "docs/troubleshooting.md"
      provides: "Operational troubleshooting and state cleanup reference"
    - path: "scripts/check-release-docs.sh"
      provides: "Docs consistency checks covering Phase 7 docs"
    - path: "crates/ctxpack/tests/release_packaging.rs"
      provides: "Rust integration test coverage for docs checker contract"
    - path: "README.md"
      provides: "Discoverable links to quickstart, agent setup, release, and troubleshooting docs"
  key_links:
    - from: "README.md"
      to: "docs/troubleshooting.md"
      via: "support/troubleshooting link"
      pattern: "docs/troubleshooting\\.md"
    - from: "scripts/check-release-docs.sh"
      to: "docs/agent-setup.md"
      via: "grep checks for proof taxonomy and unsupported Cursor/OpenCode claims"
      pattern: "deterministic protocol proof|real-client proof|Cursor|OpenCode"
    - from: "scripts/check-release-docs.sh"
      to: "docs/troubleshooting.md"
      via: "grep checks for PATH, CTXPACK_HOME, wrong-cwd, setup-check, and pack resource caveats"
      pattern: "CTXPACK_HOME|wrong cwd|setup-check|session-scoped"
---

<objective>
Add troubleshooting docs and a Phase 7 docs consistency gate.

Purpose: The phase should leave docs accurate enough for users and machine-checkable enough for Phase 8 release gates.
Output: Troubleshooting reference, README doc links, and expanded docs checker/test coverage.
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
@.planning/phases/07-documentation-troubleshooting/07-CONTEXT.md
@.planning/phases/05-release-identity-binary-packaging/VERIFICATION.md
@.planning/phases/06-agent-setup-first-pack-adoption/VERIFICATION.md
@.planning/phases/07-documentation-troubleshooting/07-documentation-troubleshooting-01-SUMMARY.md
@.planning/phases/07-documentation-troubleshooting/07-documentation-troubleshooting-02-SUMMARY.md
@README.md
@docs/release.md
@docs/quickstart.md
@docs/agent-setup.md
@scripts/check-release-docs.sh
@crates/ctxpack/tests/release_packaging.rs

<decision_trace>
- DOCS-02: Docs must cover install methods, PATH failures, absolute MCP binary paths, `CTXPACK_HOME`, uninstall/state cleanup, wrong-cwd behavior, and common MCP startup failures.
- DOCS-04: Docs must distinguish deterministic protocol proof from real-client proof and avoid unsupported Cursor/OpenCode real-client claims.
- Phase 7 decision: Extend existing docs checks if cleaner than introducing another docs test script.
- Scope guard: Scripts/tests may be edited only to check docs. Do not edit runtime source code or add new product behavior.
</decision_trace>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add troubleshooting and state cleanup reference</name>
  <files>docs/troubleshooting.md</files>
  <action>Create `docs/troubleshooting.md` with concrete symptoms, likely causes, and fixes for: `ctxpack: command not found`, GUI/agent PATH differences, using an absolute `ctxpack` binary path in MCP config, verifying `ctxpack --version` and `ctxpack --help`, `CTXPACK_HOME` location and isolated temp homes, uninstall and state cleanup (`~/.ctxpack` or custom home), wrong-cwd behavior and explicit `--repo`/MCP `repo`, common stdio MCP startup failures, stdout cleanliness expectations, permissions/read-only home behavior, `setup-check` scope, and same-session/session-scoped pack resource caveats with `get_pack` as the durable reconnect path. Keep examples source-safe and local-only; do not tell users ctxpack runs their project tests or mutates global agent config.</action>
  <verify>
    <automated>python3 -c "from pathlib import Path; d=Path('docs/troubleshooting.md').read_text(); required=['command not found','absolute','PATH','CTXPACK_HOME','~/.ctxpack','uninstall','state cleanup','wrong cwd','explicit `--repo`','MCP startup','stdout','setup-check','does not run real agent clients','session-scoped','get_pack']; missing=[s for s in required if s not in d]; assert not missing, missing"</automated>
  </verify>
  <done>Troubleshooting docs cover DOCS-02 operational failures and the DOCS-04 pack/proof caveats.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Expand docs consistency gate and README links</name>
  <files>README.md, scripts/check-release-docs.sh, crates/ctxpack/tests/release_packaging.rs</files>
  <behavior>
    - Test 1: Docs checker requires `README.md`, `docs/release.md`, `docs/quickstart.md`, `docs/agent-setup.md`, and `docs/troubleshooting.md` to exist.
    - Test 2: Docs checker requires README links to quickstart, release, agent setup, and troubleshooting docs.
    - Test 3: Docs checker requires installed-binary first-pack commands: `ctxpack init --repo`, `ctxpack setup-check --repo`, `ctxpack prepare-task`, and `ctxpack get-pack`.
    - Test 4: Docs checker requires troubleshooting coverage for PATH, absolute binary paths, `CTXPACK_HOME`, wrong cwd, MCP startup, setup-check scope, uninstall/state cleanup, and session-scoped pack resources.
    - Test 5: Docs checker requires deterministic-vs-real-client proof language and rejects unsupported Cursor/OpenCode real-client tool-call claims.
  </behavior>
  <action>Update README with a compact "More docs" or equivalent section linking `docs/quickstart.md`, `docs/release.md`, `docs/agent-setup.md`, and `docs/troubleshooting.md`. Expand `scripts/check-release-docs.sh` rather than creating a new linter: add file existence checks, required phrase checks for the new docs, and reject phrases that imply Cursor/OpenCode real-client tool-call proof or `cargo run` as the normal setup path. Update `crates/ctxpack/tests/release_packaging.rs` so `release_docs_script_contract` expects the new docs files and proof/troubleshooting checks. Keep the checker narrow and grep-based; do not turn it into a Markdown parser.</action>
  <verify>
    <automated>bash -n scripts/check-release-docs.sh && bash scripts/check-release-docs.sh && cargo test -p ctxpack --test release_packaging release_docs -- --nocapture</automated>
  </verify>
  <done>Phase 7 docs are discoverable from README and guarded by automated docs consistency checks.</done>
</task>

</tasks>

<verification>
- `bash scripts/check-release-docs.sh`
- `cargo test -p ctxpack --test release_packaging release_docs -- --nocapture`
- `cargo run -p ctxpack -- --help`
- `cargo test --workspace`
</verification>

<success_criteria>
Plan 03 is complete when DOCS-02 and DOCS-04 are satisfied by troubleshooting docs plus an automated docs gate that covers README, quickstart, release, agent setup, and troubleshooting consistency.
</success_criteria>

<output>
After completion, create `.planning/phases/07-documentation-troubleshooting/07-documentation-troubleshooting-03-SUMMARY.md`
</output>
