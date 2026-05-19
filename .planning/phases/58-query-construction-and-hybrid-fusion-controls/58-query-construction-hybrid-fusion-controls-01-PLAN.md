---
phase: 58
title: Query Construction And Hybrid Fusion Controls
status: planned
requirements_addressed:
  - QUERY-01
  - QUERY-02
  - QUERY-03
  - QUERY-04
depends_on:
  - 57
---

# Phase 58 Plan: Query Construction And Hybrid Fusion Controls

<objective>
Make query construction explicit and source-free, then use the resulting facets to control hybrid lexical, semantic, symbol, graph, test, history, precision, and memory fusion under fixed-budget evaluation variants.
</objective>

<threat_model>
- Semantic query drift hides exact identifiers, paths, or stack trace evidence.
- Fusion weights let weak semantic candidates outrank explicit user anchors.
- Query traces expose source snippets or terminal output that should remain private.
- Eval variants become incomparable because each path builds different query text.
</threat_model>

<must_haves>
- Typed query facets and query construction traces exist in shared contracts.
- Planner retrieval uses a shared query set across retrievers.
- Candidate evidence records which facets contributed to ranking.
- Anchor dominance and semantic caps are explicit and tested.
- Eval variants share the same query trace for comparable results.
</must_haves>

<tasks>

<task id="58.1" name="Add query facet and trace contracts">
<read_first>
- `crates/ctxpack-core/src/contracts.rs`
- `crates/ctxpack-compiler/src/planning.rs`
</read_first>
<action>
Add contracts for query facets, retriever query sets, query construction traces, and fusion control summaries. Keep fields source-free and bounded.
</action>
<verify>
- Add serialization tests for representative facets: explicit path, symbol, stack frame, error phrase, domain phrase, commit clue, current diff anchor.
- Run `cargo test -p ctxpack-core`.
</verify>
<acceptance_criteria>
- Query traces can be emitted without source bodies.
- Contracts preserve enough provenance to debug retrieval failures.
</acceptance_criteria>
</task>

<task id="58.2" name="Implement query construction in the compiler">
<read_first>
- `crates/ctxpack-compiler/src/planning.rs`
- `crates/ctxpack-compiler/src/search.rs`
- `crates/ctxpack-compiler/src/ranking.rs`
</read_first>
<action>
Implement a query construction module or helper that extracts facets from task text, explicit paths, active/current diff paths, stack-trace-like text, symbol-like identifiers, route/config tokens, commit-like clues, and conceptual phrases.
</action>
<verify>
- Add compiler tests for path extraction, symbol extraction, stack trace extraction, current diff anchors, and conceptual query phrases.
- Run `cargo test -p ctxpack-compiler query`.
</verify>
<acceptance_criteria>
- Existing callers still work with only raw task text.
- Explicit paths and current diff paths become hard anchors.
- Semantic phrases remain available for conceptual retrieval.
</acceptance_criteria>
</task>

<task id="58.3" name="Refactor retrievers to consume the shared query set">
<read_first>
- `crates/ctxpack-compiler/src/planning.rs`
- `crates/ctxpack-compiler/src/ranking.rs`
- `crates/ctxpack-compiler/src/eval.rs`
</read_first>
<action>
Wire lexical, semantic, symbol, history, test, memory, and graph seed generation to the shared query set. Record query facet provenance on candidate signals.
</action>
<verify>
- Add tests showing a candidate can report which query facet matched it.
- Add tests showing semantic-only matches are capped below explicit anchors.
- Run `cargo test -p ctxpack-compiler ranking`.
</verify>
<acceptance_criteria>
- Fusion behavior is deterministic for a fixed query trace.
- Anchors remain dominant over low-confidence semantic evidence.
- Candidate feature exports include query/fusion trace metadata where appropriate.
</acceptance_criteria>
</task>

<task id="58.4" name="Add hybrid fusion controls and fixed eval variants">
<read_first>
- `crates/ctxpack-compiler/src/eval.rs`
- `crates/ctxpack-core/src/contracts.rs`
- `docs/benchmarking.md`
</read_first>
<action>
Extend historical eval options to run fixed-budget variants with the same query trace: lexical, lexical+graph, lexical+semantic, precision-enriched semantic, and full hybrid. Add fusion-control summaries to reports.
</action>
<verify>
- Add eval tests using a fixed fixture manifest.
- Run the existing v2.3 eval smoke and ensure outputs remain backward compatible.
</verify>
<acceptance_criteria>
- Variant reports identify enabled retrievers and fusion controls.
- Reports can compare variants without changing query construction.
- Existing benchmark commands still work.
</acceptance_criteria>
</task>

<task id="58.5" name="Document query traces and debugging workflow">
<read_first>
- `docs/benchmarking.md`
- `docs/architecture.md`
- `.planning/ROADMAP.md`
</read_first>
<action>
Document how query traces explain retrieval outcomes and how hybrid controls prevent context drift. Update planning state after verification.
</action>
<verify>
- Run `cargo test --workspace`.
- Run `cargo run -p ctxpack -- --help`.
- Run `git diff --check`.
</verify>
<acceptance_criteria>
- Docs explain why semantic recall may not improve if a benchmark does not exercise semantic facets.
- Phase 58 is marked complete only after workspace tests and CLI help pass.
</acceptance_criteria>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack-core`
- `cargo test -p ctxpack-compiler query`
- `cargo test -p ctxpack-compiler ranking`
- `scripts/smoke-v23-eval.sh`
- `cargo test --workspace`
- `cargo run -p ctxpack -- --help`
</verification>

<success_criteria>
- Query construction is typed, inspectable, and source-free.
- All retrievers use the same query trace.
- Fusion controls protect anchors and exact evidence.
- Eval variants can compare retrieval strategies fairly.
</success_criteria>
