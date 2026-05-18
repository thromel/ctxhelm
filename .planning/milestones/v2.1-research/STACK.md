# Research: Stack for v2.1 Pack Inspector & GraphRAG Retrieval

## Question

What stack additions or changes are needed for an optional pack inspector UI and
deeper graph/embedding retrieval in the existing Rust/MCP ctxpack architecture?

## Current Baseline

ctxpack already has:

- Rust CLI and MCP crates.
- Source-free storage, eval, benchmark, feedback, memory, semantic metadata,
  precision edges, workspace manifests, and shared artifacts.
- Existing local semantic retrieval with source-free vector metadata.
- No daily UI surface; CLI/MCP remain the primary integration layer.

## Stack Recommendation

### UI Shell

Use a thin web UI first, then wrap it with Tauri only when native packaging is
worth the extra surface area.

Initial stack:

- Rust HTTP/JSON view endpoints in a small `ctxpack-inspector` or `ctxpack-ui`
  layer.
- Static web frontend for local pack/graph/report inspection.
- Cytoscape.js for graph views where compound nodes and graph layouts matter.
- Plain tables and charts for pack sections, retrieval gaps, benchmark trends,
  and signal contribution.

Tauri is a good later packaging candidate because it pairs a Rust backend with a
web frontend and uses the OS WebView instead of bundling a browser engine. Its
permission model also fits ctxpack's local-first boundary, but it adds packaging
and ACL work that should not block the first inspector.

### Graph Visualization

Use Cytoscape.js for repository/evidence graph visualization:

- It supports graph elements, styles, layouts, and headless operation.
- Compound nodes map well to workspace -> repo -> package/module grouping.
- Layout extensions let us choose simple defaults first and improve later.

React Flow is better for editable workflow diagrams than dense dependency and
evidence graphs. Since ctxpack should not become an editor, Cytoscape.js is the
better first graph library.

### Retrieval Layer

Do not import a generic GraphRAG framework directly into ctxpack.

Instead, implement code-specific GraphRAG as an internal retrieval policy:

- Keep source-free graph edges in SQLite/storage.
- Add graph neighborhoods, community/module summaries, and feedback-weighted
  graph scoring as retrieval signals.
- Use embeddings as one opt-in signal, not the primary architecture.
- Reuse benchmark and feedback reports to measure lift before promoting a
  policy.

### Embeddings

Keep local deterministic embeddings as the default. Add provider policy and
evaluation hooks before adding cloud embedding execution.

If cloud providers are added later:

- OpenAI `text-embedding-3-small` and `text-embedding-3-large` provide
  documented dimensionality and shortened-dimension controls.
- Voyage offers current general embedding and open-weight options; code-specific
  provider choices should be explicit policy options, not defaults.

## Source Notes

- Tauri ACL permissions emphasize explicit command access and scopes:
  https://v2.tauri.app/reference/acl/permission/
- Cytoscape.js supports compound nodes, graph initialization, styles, and
  layouts:
  https://js.cytoscape.org/index.html
- Microsoft GraphRAG separates local, global, DRIFT, and basic vector search:
  https://microsoft.github.io/graphrag/query/overview/
- OpenAI embedding docs describe `text-embedding-3-small`,
  `text-embedding-3-large`, dimensions, and shortening:
  https://openai.com/index/new-embedding-models-and-api-updates/
- Voyage embeddings list current model families, dimensions, and open-weight
  options:
  https://docs.voyageai.com/docs/embeddings

