# Phase 52: Source-Free Candidate Feature Export - Context

**Gathered:** 2026-05-19
**Status:** Ready for planning
**Mode:** Auto-generated infrastructure context

<domain>
## Phase Boundary

This phase turns ctxhelm's typed retrieval candidates into source-free feature rows that can be stored, inspected, compared, and used by later paired-baseline and learned-policy phases.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Build on existing `ContextPlan.retrieval_candidates` instead of duplicating retrieval logic.
- Keep feature rows source-free and path/metadata based.
- Store exports in local ctxhelm state under the repo ID, not in the repository by default.
- Add lifecycle commands for export, list, inspect, compare, and delete.
- Defer historical gold labels and learned-policy consumption to later v2.3 phases.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `RetrievalCandidate`, `RetrievalSignalScore`, and `RetrievalEvidence` already capture kind, path, role, signal scores, and source-free evidence.
- `prepare_context_plan_with_paths_history_and_semantic` already produces candidate rows covering files, tests, symbols, docs, commits, config, and memory where available.
- `ctxhelm eval feedback` already records read/edit/test/correction labels for future feedback joins.
- Phase 51 added local source-free report cache storage conventions under ctxhelm home.

### Established Patterns
- Reports and artifacts expose `sourceTextLogged: false`.
- CLI JSON contracts use camelCase.
- Management commands should support Markdown and JSON.

</code_context>

<specifics>
## Specific Ideas

- Add `CandidateFeatureExport` and `CandidateFeatureRow` contracts to `ctxhelm-core`.
- Add `ctxhelm eval features export/list/inspect/compare/delete`.
- Include signal score columns plus aggregated lexical, semantic, graph, history, test, memory, and feedback score fields.
- Include source-free labels such as `selected` and `unknown`.

</specifics>

<deferred>
## Deferred Ideas

- Historical gold/read/edit labels become richer in Phase 53 and Phase 54.
- Offline learned-policy generation belongs to Phase 54.

</deferred>
