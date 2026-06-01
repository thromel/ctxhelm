# Research: Pitfalls for v2.1 Pack Inspector & GraphRAG Retrieval

## Question

What mistakes are most likely when adding a UI inspector and deeper graph or
embedding retrieval to ctxhelm?

## Pitfalls

### Turning The UI Into The Product Center

The inspector must not become the daily coding workflow. If users need to live
inside the UI to benefit from ctxhelm, the product drifts away from the original
agent-native thesis.

Prevention:

- UI is diagnostics/export/preview only.
- CLI and MCP remain complete without UI.
- Every UI view has a JSON/CLI equivalent.

### Source Leakage Through Convenience Views

Pack inspectors are tempted to show source snippets, prompts, logs, or terminal
output. That can violate the source-free contract for reports and team
artifacts.

Prevention:

- Separate source-bearing pack snippets from source-free inspector reports.
- Keep privacy status visible on every inspector view.
- Add sentinel tests for UI/export payloads.

### Overbuilding Generic GraphRAG

Generic GraphRAG pipelines often build LLM-derived knowledge graphs over prose.
ctxhelm already has a code graph, symbol graph, tests, history, memory, and
feedback. Rebuilding another graph layer would add cost without clear lift.

Prevention:

- Start from existing code graph edges and benchmark gaps.
- Add graph communities and source-free summaries only where useful.
- Treat vector-only and graph-only baselines as comparison arms.

### Unmeasured Embedding Drift

Changing embedding provider, dimensions, or fusion weights can look better on
one task and worse on another.

Prevention:

- Store provider/dimension/model metadata.
- Require benchmark and feedback comparison before policy promotion.
- Keep semantic retrieval opt-in until evidence justifies default usage.

### UI Performance On Large Repos

Rendering full graphs for large repositories can become unusable.

Prevention:

- Render subgraphs, communities, and top candidates by default.
- Use progressive disclosure and search/filter controls.
- Keep graph layout budgeted and cache derived layouts.

### Tauri Permission Sprawl

If a desktop wrapper is added too early, broad filesystem permissions can erode
the trust story.

Prevention:

- Start with static/local web inspector.
- If Tauri is added, expose narrow read-only commands and scoped permissions.

