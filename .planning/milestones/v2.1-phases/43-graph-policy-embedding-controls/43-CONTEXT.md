# Phase 43: Graph-Aware Policy & Embedding Controls - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning
**Mode:** Autonomous inline execution

<domain>
## Phase Boundary

Phase 43 adds report-only controls for graph-aware policy experiments and local
semantic provider status. It must not silently change default ranking, enable
cloud embeddings, or introduce cloud reranking.
</domain>

<decisions>
## Implementation Decisions

1. Add source-free semantic provider status contracts.
2. Add policy experiment reports comparing lexical, local semantic, graph, and
   current default rows.
3. Keep default retrieval unchanged.
4. Keep cloud embedding/reranking flags explicit and false.
5. Expose CLI commands and smokes before future release-gate consolidation.
</decisions>

<code_context>
## Existing Code Insights

- Local semantic retrieval already uses `local_hash` and is disabled by default.
- Historical eval already supports `semantic_enabled`.
- Phase 42 now provides graph neighborhood reports.
- Team policy already has cloud permission fields defaulting to false.
</code_context>

<specifics>
## Specific Ideas

- `ctxhelm semantic status --query "..."`
- `ctxhelm eval policy experiments "..."`
- Include provider kind, model ID, dimensions, vector counts, semantic usage,
  cloud gates, and privacy flags.
</specifics>

<deferred>
## Deferred Ideas

- Learned graph-aware ranking belongs after enough report data exists.
- Cloud providers remain future explicit policy-gated work.
</deferred>
