# Phase 40: Local Web Pack Inspector - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning
**Mode:** Autonomous inline execution

<domain>
## Phase Boundary

Phase 40 builds on Phase 39's `PackInspectorView` by making the HTML export a
usable read-only diagnostic inspector. It should remain static and optional;
normal CLI/MCP workflows must not depend on a UI process.

The UI should help users inspect target files, tests, commands, evidence,
warnings, selected memory, and section budgets without exposing raw source text.
</domain>

<decisions>
## Implementation Decisions

1. Keep delivery as a static HTML export from `ctxhelm inspector export`.
2. Add client-side filters for path/reason text, source-bearing sections, and
   evidence/category views.
3. Render warnings, diagnostics, retrieval candidates, selected memory, and
   signal scores in the HTML so it becomes a true inspector rather than a
   summary table.
4. Keep styles dense and utilitarian, with stable table/card dimensions and
   narrow-screen behavior.
5. Add a script-level smoke that exports HTML/JSON from a temp repo and checks
   for UI hooks plus no source sentinel leakage.
</decisions>

<code_context>
## Existing Code Insights

- `render_pack_inspector_html` currently creates a basic static page from
  `PackInspectorView`.
- CLI support already exists through `ctxhelm inspector export`.
- Release docs currently know about docs but not an inspector smoke.
- There is no frontend build system in the repo, so Phase 40 should avoid
  adding one.
</code_context>

<specifics>
## Specific Ideas

- Add a search input and filter checkboxes/selects.
- Mark rows with `data-filter-text`, `data-source-bearing`, and `data-kind`.
- Add source-free candidate and memory tables.
- Embed compact vanilla JavaScript; no external CDN or package install.
- Add `scripts/smoke-inspector.sh` and wire it into release docs.
</specifics>

<deferred>
## Deferred Ideas

- Cytoscape graph visualization belongs to Phase 42.
- Retrieval health dashboards belong to Phase 41.
- Agent adapter previews belong to Phase 44.
</deferred>
