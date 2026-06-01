---
phase: 03-measured-retrieval-lift-eval-gates
plan: 05
type: execute
wave: 5
depends_on: [01, 02, 03, 04]
files_modified:
  - crates/ctxhelm/tests/cli_compat.rs
  - crates/ctxhelm-mcp/src/lib.rs
  - scripts/smoke-historical-eval.sh
autonomous: true
requirements: [DIAG-03, RETR-05, EVAL-04, EVAL-05]
must_haves:
  truths:
    - "CLI JSON and Markdown surfaces expose additive ranking/eval fields without removing existing compatibility keys."
    - "MCP prepare_task structuredContent carries attributed recommendations without adding new tools or changing tool names."
    - "Maintainer can run a bounded historical-eval smoke on this repo or RefactoringMiner without full-worktree checkout costs."
    - "Final validation runs workspace tests and CLI help after all Phase 3 changes."
  artifacts:
    - path: "crates/ctxhelm/tests/cli_compat.rs"
      provides: "Binary CLI compatibility checks for additive retrieval/eval fields"
    - path: "crates/ctxhelm-mcp/src/lib.rs"
      provides: "MCP compatibility checks for attributed ContextPlan structuredContent"
    - path: "scripts/smoke-historical-eval.sh"
      provides: "Bounded source-free large-repo-ready eval smoke"
  key_links:
    - from: "crates/ctxhelm/src/main.rs"
      to: "crates/ctxhelm/tests/cli_compat.rs"
      via: "compiled binary eval history and prepare-task JSON"
      pattern: "retrievalCandidates"
---

<objective>
Close Phase 3 through user-facing compatibility checks and bounded smoke validation.

Purpose: The measured ranking work must remain usable through CLI/MCP and provable on a large-history repository path without leaking source.
Output: Compatibility tests, smoke script, and final workspace validation.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-CONTEXT.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-RESEARCH.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-04-SUMMARY.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-01-SUMMARY.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-02-SUMMARY.md

<decision_trace>
- D-02: Preserve public JSON compatibility by adding fields.
- D-05: Show or explain lift at fixed budgets.
- D-06: Large real repositories are smoke tests, deterministic fixtures are the main proof.
- D-09: Reports remain source-free and prompt-free.
- Deferred: real Codex/Claude client durability is Phase 4, not this plan.
</decision_trace>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Guard CLI and MCP additive compatibility</name>
  <files>crates/ctxhelm/tests/cli_compat.rs, crates/ctxhelm-mcp/src/lib.rs</files>
  <behavior>
    - Test 1: `ctxhelm prepare-task` JSON compatibility still includes existing keys and now includes retrievalCandidates plus target/test attribution.
    - Test 2: `ctxhelm eval history --format json --budget 10` includes eval range, metrics, ablations, gap summaries, and sourceTextLogged=false.
    - Test 3: MCP tool list is unchanged and `prepare_task` structuredContent exposes attributed recommendations.
  </behavior>
  <action>Update binary and MCP compatibility tests for additive Phase 3 fields per D-02. Do not add MCP tools, resources, prompts, or client durability behavior; Phase 4 owns real client restart/wrong-repo semantics. Keep old shape assertions for targetFiles, relatedTests, riskFlags, diagnostics, privacyStatus, and existing eval fields.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test cli_compat -- --nocapture && cargo test -p ctxhelm-mcp public_surface -- --nocapture && cargo test -p ctxhelm-mcp prepare_task -- --nocapture</automated>
  </verify>
  <done>CLI and MCP compatibility tests prove Phase 3 fields are additive and existing surfaces still work.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add bounded source-free historical eval smoke script</name>
  <files>scripts/smoke-historical-eval.sh</files>
  <behavior>
    - Test 1: Script runs against `CTXHELM_SMOKE_REPO` or the current repo with a small limit and JSON output.
    - Test 2: Script accepts `CTXHELM_REFACTORINGMINER_REPO=/Users/romel/Documents/GitHub/RefactoringMiner` when present but skips clearly when absent.
    - Test 3: Script fails if the report contains source text, prompt/task text fields, or missing fixed-budget eval metadata.
    - Test 4: Script passes `--budget "$budget"` to every eval run and fails if the JSON report budget does not match.
  </behavior>
  <action>Create a portable bash smoke script for EVAL-04. Default to `CTXHELM_SMOKE_REPO=${PWD}`, `CTXHELM_SMOKE_LIMIT=3`, and `CTXHELM_SMOKE_BUDGET=10`; if `CTXHELM_REFACTORINGMINER_REPO` is set and exists, run the same bounded eval there. The script must run `cargo run -p ctxhelm -- eval history --repo "$repo" --limit "$limit" --budget "$budget" --format json`, validate source-free fields and matching budget metadata using `python3` or `ruby` available on macOS, and print a compact summary. It must not checkout entire external worktrees beyond the eval implementation, run project tests, upload data, or require network access.</action>
  <verify>
    <automated>CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_LIMIT=2 CTXHELM_SMOKE_BUDGET=10 bash scripts/smoke-historical-eval.sh</automated>
  </verify>
  <done>Maintainer has a repeatable bounded smoke path for current repo and optional RefactoringMiner verification.</done>
</task>

<task type="auto">
  <name>Task 3: Run final Phase 3 validation</name>
  <files>None</files>
  <action>Run the full validation gate after Tasks 1-2: workspace tests, CLI help, and a small eval history smoke. This task is validation-only. If a validation command exposes a bug, stop and report the blocker with the owning earlier plan/task that must be reopened; do not hide missing implementation inside the final validation plan. Do not add cloud/vector features, parser/runtime migrations, or Phase 4 client durability.</action>
  <verify>
    <automated>cargo test --workspace && cargo run -p ctxhelm -- --help && CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_LIMIT=2 CTXHELM_SMOKE_BUDGET=10 bash scripts/smoke-historical-eval.sh</automated>
  </verify>
  <done>All Phase 3 code paths pass workspace validation, CLI help validation, and bounded source-free historical eval smoke.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm --test cli_compat -- --nocapture`
- `cargo test -p ctxhelm-mcp public_surface -- --nocapture`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
- `CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_LIMIT=2 CTXHELM_SMOKE_BUDGET=10 bash scripts/smoke-historical-eval.sh`
</verification>

<success_criteria>
Plan 05 is complete when Phase 3 remains compatible through CLI/MCP, has a bounded large-repo-ready eval smoke, and passes the full project validation gate.
</success_criteria>

<output>
After completion, create `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-05-SUMMARY.md`
</output>
