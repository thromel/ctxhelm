# Phase 42: Graph Neighborhoods & Communities - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning
**Mode:** Autonomous inline execution

<domain>
## Phase Boundary

Phase 42 adds source-free graph neighborhood reports. These reports should show
how files, tests, dependency edges, memory cards, and feedback signals cluster
around a task or explicit anchors.

The graph must be budgeted and non-recursive by default. It should summarize
relationships, not pull raw source into another artifact.
</domain>

<decisions>
## Implementation Decisions

1. Add graph contracts in `ctxpack-core`.
2. Build graph neighborhoods in `ctxpack-compiler` from existing safe
   dependency edges, related tests, memory cards, feedback events, and optional
   task planning.
3. Add a CLI command for source-free JSON/Markdown graph reports.
4. Keep max node/edge limits explicit and add diagnostics when capped.
5. Defer graph visualization to the inspector/GraphRAG policy phases.
</decisions>

<code_context>
## Existing Code Insights

- Dependency edges are already source-free and safe-inventory backed.
- Related tests already provide source-to-test links and commands.
- Memory cards and feedback events are stored as source-free metadata.
- Workspace metadata exists, but Phase 42 can start with single-repo graph
  neighborhoods and keep workspace expansion additive later.
</code_context>

<specifics>
## Specific Ideas

- `GraphNeighborhoodReport` with nodes, edges, communities, diagnostics, and
  `sourceTextLogged: false`.
- `ctxpack graph neighborhood "task" --path src/auth/session.ts`.
- Support task-derived anchors when no path is provided.
- Community summaries by first path segment or role/package label.
</specifics>

<deferred>
## Deferred Ideas

- Cytoscape rendering belongs to later inspector work.
- Graph-aware ranking policy experiments belong to Phase 43.
</deferred>
