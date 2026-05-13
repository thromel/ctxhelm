---
phase: 02-trust-layer-operational-diagnostics
plan: 04
type: execute
wave: 4
depends_on: ["02-03"]
files_modified:
  - crates/ctxpack-compiler/src/lib.rs
  - crates/ctxpack-compiler/src/planning.rs
  - crates/ctxpack-compiler/src/packs.rs
  - crates/ctxpack-compiler/src/cards.rs
autonomous: true
requirements: [SAFE-01, SAFE-02, SAFE-04, SAFE-05, DIAG-01, DIAG-04]
must_haves:
  truths:
    - "Context plans expose structured diagnostics for stale cache, low-information tasks, partial signals, and source-read gaps."
    - "Context packs revalidate every source-bearing snippet path against current safe inventory immediately before reading."
    - "Context cards are generated from fresh safe inventory and remain source-free while reporting degraded inputs."
  artifacts:
    - path: "crates/ctxpack-compiler/src/planning.rs"
      provides: "Plan-level diagnostics and riskFlags compatibility projection"
      contains: "diagnostics"
    - path: "crates/ctxpack-compiler/src/packs.rs"
      provides: "Pack snippet revalidation through index policy"
      contains: "read_safe_source"
    - path: "crates/ctxpack-compiler/src/cards.rs"
      provides: "Fresh inventory and diagnostics for source-free cards"
      contains: "load_or_refresh_inventory"
  key_links:
    - from: "crates/ctxpack-compiler/src/planning.rs"
      to: "crates/ctxpack-index/src/search.rs"
      via: "diagnostic-aware retrieval reports"
      pattern: "diagnostics"
    - from: "crates/ctxpack-compiler/src/packs.rs"
      to: "crates/ctxpack-index/src/policy.rs"
      via: "safe snippet reads"
      pattern: "read_safe_source"
    - from: "crates/ctxpack-compiler/src/cards.rs"
      to: "crates/ctxpack-index/src/freshness.rs"
      via: "fresh safe inventory"
      pattern: "load_or_refresh_inventory"
---

<objective>
Make compiler outputs explain degraded plans and revalidate all source-bearing pack/card reads.

Purpose: SAFE-04 is specifically about pack, file-resource, and card source reads. DIAG-01 and DIAG-04 require users and maintainers to distinguish weak task input from stale, skipped, or partial subsystem behavior.
Output: Diagnostic-rich `ContextPlan`/`ContextPack` values, safe snippet revalidation, and deterministic weak-plan/revalidation tests.
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
@.planning/codebase/ARCHITECTURE.md
@.planning/codebase/CONCERNS.md
@.planning/codebase/TESTING.md
@AGENTS.md

<interfaces>
Existing compiler APIs to preserve:
```rust
pub fn prepare_context_plan(repo_root: impl AsRef<Path>, task: &str, task_type: TaskType)
    -> Result<ContextPlan, InventoryError>;

pub fn compile_context_pack_from_plan_for_agent(
    repo_root: impl AsRef<Path>,
    task: &str,
    plan: &ContextPlan,
    budget: PackBudget,
    target_agent: Option<&str>,
) -> ContextPack;
```

Plan 03 provides diagnostic-aware index reports and `read_safe_source`.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Surface plan diagnostics while preserving riskFlags</name>
  <files>crates/ctxpack-compiler/src/lib.rs, crates/ctxpack-compiler/src/planning.rs</files>
  <read_first>
    - `crates/ctxpack-compiler/src/planning.rs`
    - `crates/ctxpack-compiler/src/lib.rs`
    - `crates/ctxpack-index/src/search.rs`
    - `crates/ctxpack-index/src/git.rs`
  </read_first>
  <behavior>
    - DIAG-01: context plans include structured diagnostics for `low_information_task`, stale/rebuilt/cache status, missing git/git timeout, skipped files, parse gaps, and partial graph/test/history coverage.
    - DIAG-04: weak-plan scenarios like `Fixes #1061`, empty repos, and missing-git history are covered by deterministic tests.
    - DIAG-02 compatibility precursor: warning/error diagnostics are projected into existing `risk_flags` so older clients still see risk-like information.
  </behavior>
  <action>
    Extend planning fusion to merge index diagnostics into `ContextPlan.diagnostics`. Convert existing low-information and unavailable path/risk-flag logic into diagnostics first, then project warning/error diagnostics into `risk_flags` without removing existing risk messages that compatibility tests expect. Keep `missing_info_questions` for low-information tasks. Do not alter candidate scoring or introduce Phase 3 typed ranking work.
  </action>
  <verify>
    <automated>cargo test -p ctxpack-compiler diagnostics -- --nocapture</automated>
    <automated>cargo test -p ctxpack-compiler low_information -- --nocapture</automated>
    <automated>cargo test -p ctxpack-compiler unavailable -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - Plans for low-information tasks include both `diagnostics` and `missingInfoQuestions`.
    - Existing risk flag compatibility tests continue to pass.
    - Compiler tests prove diagnostics are source-free.
  </acceptance_criteria>
  <done>Context plans explain weak/degraded conditions through diagnostics while preserving `riskFlags` compatibility.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Revalidate pack snippets and source-free cards</name>
  <files>crates/ctxpack-compiler/src/lib.rs, crates/ctxpack-compiler/src/packs.rs, crates/ctxpack-compiler/src/cards.rs</files>
  <read_first>
    - `crates/ctxpack-compiler/src/packs.rs`
    - `crates/ctxpack-compiler/src/cards.rs`
    - `crates/ctxpack-index/src/policy.rs`
    - `crates/ctxpack-index/src/freshness.rs`
  </read_first>
  <behavior>
    - SAFE-04: pack snippets are read only after revalidating each target/test path against the current safe inventory.
    - SAFE-04: card generation uses fresh safe inventory and does not embed source snippets.
    - SAFE-05/DIAG-01: deleted, newly sensitive, newly generated, non-UTF-8, oversized, or unreadable snippet paths produce diagnostics/warnings instead of empty snippets.
    - DIAG-04: deterministic tests cover plan path becomes `.env`, moved under `dist/`, deleted after plan, and oversized/non-UTF-8 snippet candidates.
  </behavior>
  <action>
    Replace direct file reads in pack rendering/snippet extraction with `load_or_refresh_inventory` plus `read_safe_source`. Add pack/card diagnostics to returned `ContextPack` and warnings text, preserving Markdown sections and existing JSON fields. Ensure generated context cards continue to be source-free summaries and report skipped/degraded inputs using diagnostics rather than contents. Do not add retrieval ranking or historical eval failure grouping.
  </action>
  <verify>
    <automated>cargo test -p ctxpack-compiler pack -- --nocapture</automated>
    <automated>cargo test -p ctxpack-compiler revalidates -- --nocapture</automated>
    <automated>cargo test -p ctxpack-compiler cards -- --nocapture</automated>
    <automated>cargo test -p ctxpack-compiler diagnostics -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - Packs never include snippets for paths that are no longer safe at read time.
    - Pack diagnostics/warnings explain why a snippet was skipped.
    - Cards remain source-free and deterministic while using fresh safe inventory.
  </acceptance_criteria>
  <done>Compiler pack/card reads are fresh, privacy-gated, diagnostic, and still source-free where required.</done>
</task>

</tasks>

<verification>
```bash
cargo test -p ctxpack-compiler diagnostics -- --nocapture
cargo test -p ctxpack-compiler low_information -- --nocapture
cargo test -p ctxpack-compiler unavailable -- --nocapture
cargo test -p ctxpack-compiler pack -- --nocapture
cargo test -p ctxpack-compiler revalidates -- --nocapture
cargo test -p ctxpack-compiler cards -- --nocapture
cargo test -p ctxpack-compiler
```
</verification>

<success_criteria>
SAFE-04 is implemented for compiler-owned pack and card source reads, DIAG-01 explains weak/degraded plans, and DIAG-04 fixtures cover weak plan and revalidation scenarios.
</success_criteria>

<output>
After completion, create `.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-04-SUMMARY.md`.
</output>
