---
phase: 03-measured-retrieval-lift-eval-gates
plan: 02
type: execute
wave: 2
depends_on: [01]
files_modified:
  - crates/ctxpack-compiler/src/ranking.rs
  - crates/ctxpack-compiler/src/planning.rs
  - crates/ctxpack-compiler/src/lib.rs
autonomous: true
requirements: [RETR-01, RETR-02, RETR-03, RETR-04, PARS-01]
must_haves:
  truths:
    - "Context planning ranks typed candidates before writing targetFiles and relatedTests."
    - "Dependency edges, related tests, co-change hints, symbols, lexical matches, anchors, and current-diff anchors can affect selected targets."
    - "Docs, commit, and config candidates are materialized as typed candidates with source-free evidence, not only folded into file scores."
    - "Graph and test expansion is one-hop and consumes the same target/test budget as lexical candidates."
    - "Every recommended target file and related test has source-free attribution."
  artifacts:
    - path: "crates/ctxpack-compiler/src/ranking.rs"
      provides: "Candidate collection, signal fusion, one-hop expansion, and fixed-budget selection"
    - path: "crates/ctxpack-compiler/src/planning.rs"
      provides: "ContextPlan projection from ranked candidates"
  key_links:
    - from: "crates/ctxpack-compiler/src/planning.rs"
      to: "crates/ctxpack-compiler/src/ranking.rs"
      via: "collect signals -> rank candidates -> project targetFiles/relatedTests"
      pattern: "rank_.*candidates"
---

<objective>
Replace ad hoc plan ordering with a typed candidate ranking pass.

Purpose: Retrieval lift must come from better ranked evidence under fixed budgets, not from appending more context.
Output: A compiler ranking module wired into `prepare_context_plan_with_paths_and_history`.
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
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-01-SUMMARY.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-03-SUMMARY.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-04-SUMMARY.md

<decision_trace>
- D-01: Typed candidate layer before ContextPlan projection.
- D-03: Attribution must be source-free.
- D-04: Graph expansion must be budgeted, shallow, and non-recursive by default.
- D-11: Do not start broad Tree-sitter, Tantivy, SQLite, rayon, notify, or MCP SDK migrations.
</decision_trace>

<interfaces>
Existing signal APIs:
```rust
lexical_search_report(repo_root, task, &SearchOptions { limit }) -> SearchReport
symbol_search_report(repo_root, task, &SymbolOptions { limit }) -> SymbolSearchReport
related_tests_report(repo_root, &source_target_paths) -> RelatedTestsReport
co_change_hints_report(repo_root, &source_target_paths, &CoChangeOptions { limit }) -> CoChangeReport
related_dependency_edges_report(repo_root, &source_target_paths, &DependencyOptions { limit }) -> DependencyEdgesReport
```

Existing planning limit:
```rust
pub(crate) const PREPARE_TASK_TARGET_LIMIT: usize = 8;
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement deterministic candidate scoring and one-hop expansion</name>
  <files>crates/ctxpack-compiler/src/ranking.rs, crates/ctxpack-compiler/src/lib.rs</files>
  <behavior>
    - Test 1: Multiple signals for the same path merge into one candidate with per-signal scores and evidence.
    - Test 2: One-hop dependency/test/history expansion adds neighbor candidates but does not recursively expand from those neighbors.
    - Test 3: `select_ranked_candidates` returns at most the requested file and test budgets and uses deterministic path tie-breaks.
    - Test 4: Ranking fixtures produce typed doc, commit, and config candidates with source-free evidence when those signals are present.
  </behavior>
  <action>Create `ranking.rs` as an internal compiler module per D-01/D-04. Define private ranking inputs around existing index result structs and emit core `RetrievalCandidate` values plus selected target/test projections. Include signal weights for anchor/currentDiff, symbol, lexical, dependency, relatedTest, coChange/history, docs/config role boosts, and explicit reason codes. Materialize docs, commits, and config as their own typed candidate kinds when those signals are available; they may also influence file/test ranking, but they must not disappear into file-only scores. Keep all evidence source-free per D-03; do not carry search snippets, commit subjects, task strings, or symbol signatures.</action>
  <verify>
    <automated>cargo test -p ctxpack-compiler ranking -- --nocapture</automated>
  </verify>
  <done>Ranking unit tests prove signal merge, one-hop expansion, fixed budgets, and deterministic ordering.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Wire ranking into context planning while preserving compatibility</name>
  <files>crates/ctxpack-compiler/src/planning.rs, crates/ctxpack-compiler/src/ranking.rs, crates/ctxpack-compiler/src/lib.rs</files>
  <behavior>
    - Test 1: A dependency neighbor that lacks lexical matches can become a ranked target when connected to a strong seed.
    - Test 2: A related test can be recommended with attribution and validation command without increasing target file budget.
    - Test 3: Explicit path anchors, including current-diff paths supplied through existing `--current-diff` plumbing, receive anchor/currentDiff attribution.
    - Test 4: Existing riskFlags for co-change and dependency evidence remain present for backward compatibility.
    - Test 5: `ContextPlan.retrievalCandidates` includes file, test, symbol, doc, commit, and config candidates in a deterministic source-free order when fixtures provide those signals.
  </behavior>
  <action>Refactor `prepare_context_plan_with_paths_and_history` to gather reports first, pass them into `ranking.rs`, and project selected candidates into `target_files`, `related_tests`, `recommended_commands`, and `retrieval_candidates`. Keep `targetFiles` and `relatedTests` shapes additive per D-02 from Plan 01. Keep diagnostics aggregation and riskFlag projection from Phase 2. Do not change MCP tool names or add parser/runtime dependencies per D-11.</action>
  <verify>
    <automated>cargo test -p ctxpack-compiler prepare_context_plan -- --nocapture</automated>
  </verify>
  <done>Context planning uses ranked candidates, graph/test/history/current-diff signals affect selection, and recommendations carry source-free attribution.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack-compiler ranking -- --nocapture`
- `cargo test -p ctxpack-compiler prepare_context_plan -- --nocapture`
- `cargo test -p ctxpack-compiler`
</verification>

<success_criteria>
Plan 02 is complete when `prepare_context_plan` ranks typed candidates under fixed budgets and projects attributed target/test recommendations without breaking existing diagnostics or riskFlags.
</success_criteria>

<output>
After completion, create `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-02-SUMMARY.md`
</output>
