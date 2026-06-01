# Research: Architecture for v2.1 Pack Inspector & GraphRAG Retrieval

## Question

How do the v2.1 features integrate with the current ctxhelm architecture?

## Proposed Architecture

```text
ctxhelm-core
  typed inspector, graph policy, and provider contracts

ctxhelm-index
  graph community metadata
  semantic provider metadata
  source-free graph/embedding status

ctxhelm-compiler
  graph-aware retrieval policy experiments
  signal contribution summaries
  inspector view-model compilation

ctxhelm-mcp
  optional resources for inspector-ready pack/report metadata
  no new broad tool surface by default

ctxhelm / ctxhelm-inspector
  CLI commands for local inspector server/export
  static local web UI or later Tauri wrapper
```

## Data Flow

```text
safe repo inventory
  -> symbols / dependencies / tests / history / memory / feedback
  -> graph communities and signal summaries
  -> context plan and pack
  -> inspector view model
  -> local UI
```

## GraphRAG Shape For Code

Use the GraphRAG idea, not the generic document implementation:

- Local code search maps to entity/symbol/file neighborhoods.
- Global search maps to module/package/workspace community summaries.
- DRIFT-like expansion maps to starting from a task, selecting a graph
  community, then refining into concrete files/tests.
- Basic vector search remains a baseline signal for comparison.

This should reuse ctxhelm's existing graph, memory, benchmark, and feedback
systems rather than adding a separate graph database or LLM-built knowledge
graph.

## Inspector View Models

Add source-free typed view models before UI code:

- `PackInspectorView`
- `CandidateEvidenceView`
- `RetrievalHealthReport`
- `GraphNeighborhoodView`
- `EmbeddingProviderStatus`
- `AgentPreview`

These contracts keep the UI thin and make CLI/MCP export possible.

## Build Order

1. Inspector contracts and static HTML/JSON export.
2. Local inspector server or static web UI reading those contracts.
3. Graph neighborhood/community reports.
4. Graph-aware retrieval policy experiments behind flags.
5. Embedding provider policy/status expansion.
6. Optional Tauri wrapper only after the web inspector proves useful.

## Architectural Trade-Offs

- Tauri now vs later: later is safer; first prove inspector data contracts.
- Cytoscape vs React Flow: Cytoscape fits dense dependency/evidence graphs;
  React Flow fits editable flows.
- GraphRAG framework vs internal graph policy: internal policy fits source-free
  code constraints and avoids LLM-built graph cost.
- Cloud embeddings now vs provider policy first: provider policy first preserves
  trust and gives evaluation gates.

