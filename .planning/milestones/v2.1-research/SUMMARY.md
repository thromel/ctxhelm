# Research Summary: v2.1 Pack Inspector & GraphRAG Retrieval

## Stack Additions

- Start with typed inspector view models and a local/static web inspector.
- Use Cytoscape.js for graph/evidence visualization.
- Defer Tauri packaging until the inspector proves useful.
- Keep local deterministic embeddings as default; add cloud provider controls
  only behind explicit policy and evaluation gates.

## Feature Table Stakes

- Pack inspector for target files, snippets, tests, validation commands,
  warnings, privacy status, budgets, selected memory, and evidence attribution.
- Retrieval-health views for benchmark trends, repeated gap families, signal
  contribution, token ROI, and workspace routing quality.
- GraphRAG-style code retrieval using existing graph edges, communities,
  semantic metadata, memory cards, feedback, and workspace boundaries.
- Agent preview views for Codex, Claude Code, Cursor, OpenCode, and generic MCP.

## Watch Outs

- Do not make the UI the daily coding surface.
- Do not leak source through reports, shared artifacts, graph labels, or UI
  exports.
- Do not import a generic GraphRAG framework before proving lift on ctxpack's
  code-specific graph.
- Do not promote new embedding providers or dimensions without benchmark and
  feedback evidence.
- Do not render full large-repo graphs by default.

## Recommended Milestone Shape

1. Inspector data contracts and CLI/static export.
2. Local web inspector with pack and retrieval-health views.
3. Graph neighborhood/community reports.
4. Graph-aware retrieval policy experiments and eval comparison.
5. Embedding provider policy/status expansion and release gates.

## Sources

- Microsoft GraphRAG query modes: https://microsoft.github.io/graphrag/query/overview/
- Cytoscape.js graph, compound node, and layout docs: https://js.cytoscape.org/index.html
- Tauri permission model: https://v2.tauri.app/reference/acl/permission/
- OpenAI embedding model and dimension controls: https://openai.com/index/new-embedding-models-and-api-updates/
- Voyage embedding model families and dimensions: https://docs.voyageai.com/docs/embeddings

