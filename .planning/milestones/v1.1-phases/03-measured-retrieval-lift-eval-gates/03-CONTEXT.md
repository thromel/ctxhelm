# Phase 3: Measured Retrieval Lift & Eval Gates - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase turns ctxhelm retrieval from useful heuristics into a measured, attributed ranking system. It should rank typed evidence from graph, test, history, symbol, current-diff, docs, config, and lexical signals before projecting to the existing `ContextPlan`, then prove whether that ranking improves over lexical retrieval at fixed budgets.

The phase must also make historical evals more decision-grade: frozen ranges, reproducible refs, role-aware labels, rename/delete handling, ranking metrics, signal ablations, source-free gap summaries, and large-repo smoke coverage such as RefactoringMiner when practical.

This phase does not own real client durability, MCP pack persistence, or cloud/vector retrieval. Those belong to later phases or future milestones.

</domain>

<decisions>
## Implementation Decisions

### Candidate Model And Ranking
- Introduce a typed candidate layer before `ContextPlan` projection rather than attaching more ad hoc fields directly to target files.
- Preserve public JSON compatibility where possible by adding fields instead of replacing existing `targetFiles`, `relatedTests`, `riskFlags`, diagnostics, or provenance shapes.
- Keep recommended-file and related-test attribution source-free: path, role, signal kind, scores, edge labels, commit ids, counts, and reason codes are acceptable; source snippets and prompt text are not.
- Make graph expansion budgeted, shallow, and non-recursive by default. Lift must come from better ranking, not larger packs.

### Evaluation
- Evaluate retrieval changes against a lexical baseline at fixed budgets. If a slice does not show lift, the report should explain the signal gaps instead of hiding the result.
- Historical eval fixtures should be deterministic and small enough for unit/integration tests, with large real repositories used as smoke tests rather than the only proof.
- Frozen eval ranges should record base/head refs, limit, mode, budget, repo identity, and effective filters so results can be reproduced.
- Rename, delete, historical-only files, generated files, sensitive files, tests, configs, docs, and source files should be labeled explicitly in eval outputs.

### Parser And Runtime Scope
- Parser/runtime upgrades are allowed only when they support observed retrieval gaps or are introduced behind existing contracts with tests.
- Do not start a broad Tree-sitter, Tantivy, SQLite, rayon, notify, or MCP SDK migration in this phase unless the eval evidence requires it.

### Product Positioning
- The user-facing value is measured context quality: why this file/test was ranked, which signal selected it, and whether it beats lexical baseline at the same budget.
- The system should remain local-first, read-only, and agent-native.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ctxhelm-core/src/contracts.rs` already owns public contracts for context plans, packs, diagnostics, provenance, privacy status, eval traces, and related command output.
- `crates/ctxhelm-index/src/lib.rs` is the stable index facade. Phase 2 added freshness reports, source-read policy, diagnostic-aware search/symbol/test/dependency/git/current-diff paths, and safe source reads.
- `crates/ctxhelm-compiler/src/planning.rs` currently fuses search, symbols, related tests, co-change hints, dependencies, current diff, and diagnostics into `ContextPlan`.
- `crates/ctxhelm-compiler/src/eval.rs` owns historical retrieval eval behavior and lexical baseline comparison.
- `crates/ctxhelm/src/main.rs` renders CLI outputs and eval/checklist reports.
- `crates/ctxhelm-mcp/src/` exposes the small MCP tool/resource/prompt surface and should consume additive plan/pack fields without widening the tool list.

### Established Patterns
- Add focused failing tests before behavior changes, especially when touching public JSON shape, privacy, diagnostics, retrieval ranking, and eval output.
- Keep crate roots as stable facades and put implementation into focused private modules.
- Use typed contracts and serde structs over stringly typed output.
- Keep source-free traces and eval reports; safe packs may contain snippets only after fresh policy revalidation.
- Compatibility is protected by binary CLI tests, JSON shape tests, MCP tests, and workspace validation.

### Integration Points
- Context planning output must still feed CLI `prepare-task`, MCP `prepare_task`, pack compilation, context cards, and eval traces.
- Eval changes should work with existing `ctxhelm eval history --repo . --limit <n> --mode <mode>` workflows.
- RefactoringMiner remains the preferred large-history smoke when practical, but deterministic local fixtures should catch core regressions.

</code_context>

<specifics>
## Specific Ideas

- Add candidate types for files, tests, symbols, docs, commits, config, and diffs with per-signal scores such as lexical, symbol, dependency, related-test, co-change, current-diff, history, docs, config, and active/anchor.
- Add source-free evidence records explaining signal contributions and ranking decisions.
- Add fixed-budget candidate selection so graph expansion cannot silently increase the compared context budget.
- Add eval ablations that can disable signals and compare combined ranking against lexical-only ranking.
- Add gap summaries grouped by path role, missing signal family, and repeated missing-file family.
- Add rename/delete/historical-only handling where possible from git diff metadata.
- Keep any report readable enough to guide the next retrieval improvement without exposing source text.

</specifics>

<deferred>
## Deferred Ideas

- Real Codex CLI and Claude Code client durability, MCP restart behavior, wrong-repo checks, and durable pack-resource semantics are Phase 4.
- Cloud embeddings, cloud reranking, local vector search, and hosted/team features remain out of scope for this milestone.
- A visual eval dashboard is out of scope; source-free CLI/JSON reports are enough for this phase.

</deferred>
