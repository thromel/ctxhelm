---
phase: 03-measured-retrieval-lift-eval-gates
plan: 04
type: execute
wave: 4
depends_on: [02, 03]
files_modified:
  - crates/ctxhelm-compiler/src/eval.rs
  - crates/ctxhelm-compiler/src/lib.rs
  - crates/ctxhelm/src/main.rs
autonomous: true
requirements: [DIAG-03, RETR-05, EVAL-03, EVAL-05, PARS-02, PARS-03]
must_haves:
  truths:
    - "Historical eval compares ctxhelm ranking, lexical-only ranking, and signal ablations over the same frozen commits and same budget."
    - "Reports include Recall@K, Precision@K, MRR@K or equivalent ranking quality, role-aware recall, test recommendation rate, lexical baseline, and lift."
    - "Retrieval failures are grouped by path role, signal gap, and repeated missing-file family without source or prompt text."
    - "Parser/runtime improvements remain gated by observed gaps and measured before/after metrics."
  artifacts:
    - path: "crates/ctxhelm-compiler/src/eval.rs"
      provides: "Ranking metrics, ablations, fixed-budget comparison, and gap summaries"
    - path: "crates/ctxhelm/src/main.rs"
      provides: "Source-free Markdown rendering for metrics, ablations, checklist failures, and gaps"
  key_links:
    - from: "crates/ctxhelm-compiler/src/eval.rs"
      to: "crates/ctxhelm-compiler/src/ranking.rs"
      via: "same candidate ranking path for combined and ablated eval runs"
      pattern: "ablation"
---

<objective>
Turn historical eval from recall snapshots into decision-grade ranking reports.

Purpose: Maintainers need to see whether ctxhelm beats lexical retrieval at fixed budgets and, if not, which signal families failed.
Output: Metrics, ablations, lift, and source-free retrieval gap reports.
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
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-02-SUMMARY.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-03-SUMMARY.md

<decision_trace>
- D-05: Evaluate retrieval changes against lexical baseline at fixed budgets; if no lift, explain signal gaps.
- D-06: Deterministic fixtures first, large repositories as smokes.
- D-09: Eval reports stay source-free and prompt-free.
- D-11: No broad parser/runtime migrations unless eval evidence requires them.
</decision_trace>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add fixed-budget ranking metrics and lexical comparison</name>
  <files>crates/ctxhelm-compiler/src/eval.rs, crates/ctxhelm-compiler/src/lib.rs</files>
  <behavior>
    - Test 1: Combined and lexical-only rankings are evaluated at the same K and same commit sample set.
    - Test 2: Report includes Recall@K, Precision@K, MRR@K, ctxhelm lift, role-aware recall, and test recommendation rate.
    - Test 3: Average recommended context files cannot increase silently when budget is fixed.
  </behavior>
  <action>Introduce source-free metric structs such as `RankingMetrics`, `EvalComparison`, and `SignalAblationResult`. Compute metrics over `recommendedContextFiles` truncated to the configured budget and lexical baseline truncated to the same budget per D-05. Keep existing legacy recall fields as compatibility projections if needed, but make the new metric records the decision-grade surface.</action>
  <verify>
    <automated>cargo test -p ctxhelm-compiler ranking_metrics -- --nocapture</automated>
  </verify>
  <done>Eval reports prove fixed-budget lexical comparison and ranking quality with deterministic tests.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add signal ablations and grouped retrieval gap summaries</name>
  <files>crates/ctxhelm-compiler/src/eval.rs, crates/ctxhelm-compiler/src/lib.rs, crates/ctxhelm/src/main.rs</files>
  <behavior>
    - Test 1: Ablation results disable one signal family at a time over the same frozen range id and commit count.
    - Test 2: Gap summaries group misses by role, missing signal family, and repeated path family.
    - Test 3: CLI `eval history` accepts or defaults a fixed budget and passes it into HistoricalEvalOptions.
    - Test 4: Markdown and JSON reports contain reason codes and counts but no commit subjects, prompt text, source snippets, or symbol signatures.
    - Test 5: Parser/runtime gate notes are derived from observed gaps and no new parser/runtime dependency is added.
  </behavior>
  <action>Add ablation execution for lexical-only, combined, and disabled-signal variants using the same frozen sample set per D-05. Add `RetrievalGapSummary` records for DIAG-03 and EVAL-05, grouped by `FileRole`, signal gap reason, and path family. Update `EvalHistoryArgs` and `render_historical_eval_report` so the CLI exposes the fixed budget used in the report. Include a test or validation assertion that Phase 3 did not add `tantivy`, `rayon`, `rusqlite`/SQLite, `notify`, broad `tree-sitter`, or an MCP SDK dependency per D-11; future parser-backed improvements must be justified by these gaps per PARS-02/PARS-03.</action>
  <verify>
    <automated>cargo test -p ctxhelm-compiler ablation -- --nocapture && cargo test -p ctxhelm historical_eval_report -- --nocapture && cargo tree --workspace --depth 1 > /tmp/ctxhelm-phase3-cargo-tree.txt && ! rg "tantivy|rayon|rusqlite|notify|tree-sitter|mcp-sdk" /tmp/ctxhelm-phase3-cargo-tree.txt</automated>
  </verify>
  <done>Reports explain lift or lack of lift through comparable ablations and source-free grouped gaps, with parser/runtime scope still gated.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Add checklist retrieval-failure summaries for DIAG-03</name>
  <files>crates/ctxhelm-compiler/src/eval.rs, crates/ctxhelm/src/main.rs</files>
  <behavior>
    - Test 1: `eval checklist` / checklist rendering includes source-free retrieval failures grouped by path role, signal gap, and repeated missing-file family.
    - Test 2: Checklist summaries reuse the same `RetrievalGapSummary` data as historical eval instead of inventing a separate string-only report.
    - Test 3: Checklist JSON and Markdown contain paths, roles, reason codes, and counts but no prompt text, commit subjects, snippets, or symbol signatures.
  </behavior>
  <action>Wire the grouped retrieval gap summaries into checklist reporting, not just `eval history`, so DIAG-03's "historical eval and checklist outputs" requirement is explicitly satisfied. Prefer a shared renderer/helper such as `render_eval_checklist` that consumes typed source-free gap records. Keep the checklist report additive and preserve any existing checklist fields.</action>
  <verify>
    <automated>cargo test -p ctxhelm eval_checklist -- --nocapture</automated>
  </verify>
  <done>Checklist outputs explain retrieval failures with the same source-free grouped failure data as historical eval reports.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm-compiler ranking_metrics -- --nocapture`
- `cargo test -p ctxhelm-compiler ablation -- --nocapture`
- `cargo test -p ctxhelm eval_checklist -- --nocapture`
- `cargo test -p ctxhelm historical_eval_report -- --nocapture`
</verification>

<success_criteria>
Plan 04 is complete when historical eval reports can justify retrieval changes with fixed-budget metrics, ablations, and source-free gap summaries.
</success_criteria>

<output>
After completion, create `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-04-SUMMARY.md`
</output>
