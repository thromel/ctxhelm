# Phase 39: Inspector Contracts & Static Export - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning
**Mode:** Autonomous inline execution

<domain>
## Phase Boundary

Phase 39 creates the source-free inspector contract layer for v2.1. The goal is
not to build the full web UI yet. The goal is to expose typed metadata views of
`ContextPlan` and `ContextPack` decisions so later phases can render,
filter, and evaluate packs without reading raw source snippets.

The inspector must preserve ctxhelm's product boundary:
- local-first by default
- read-only
- agent-native
- source-bearing pack snippets explicitly separated from source-free report
  metadata
</domain>

<decisions>
## Implementation Decisions

1. Add stable typed contracts in `ctxhelm-core` for `PackInspectorView` and its
   child records.
2. Build the inspector from an existing `ContextPlan` plus `ContextPack` so the
   view keeps links to task, pack, candidates, tests, commands, diagnostics,
   warnings, and privacy metadata.
3. Inspector records must not include `PackSection.content`; section metadata
   can include title, kind, token estimate, and whether the section is
   source-bearing.
4. Static exports should support JSON plus lightweight Markdown/HTML previews
   from the CLI. HTML is intentionally static and read-only.
5. Tests should include a sentinel proving raw source text from pack snippets
   does not leak into the inspector view.
</decisions>

<code_context>
## Existing Code Insights

- Contracts live in `crates/ctxhelm-core/src/contracts.rs`.
- Pack compilation and Markdown rendering live in
  `crates/ctxhelm-compiler/src/packs.rs`.
- CLI subcommands are centralized in `crates/ctxhelm/src/main.rs`.
- Existing pack generation can include source-bearing sections
  `target_snippets` and `test_snippets`; inspector metadata must mark those
  sections without copying their content.
</code_context>

<specifics>
## Specific Ideas

- Add `source_text_logged: false` directly to the inspector view.
- Add `source_bearing_section_count` and per-section `source_bearing`.
- Add `privacy_label`-style metadata through existing `PrivacyStatus`.
- Add `ctxhelm inspector export <task> --format json|markdown|html`.
- Keep `PackFormat` unchanged to avoid touching unrelated command surfaces.
</specifics>

<deferred>
## Deferred Ideas

- Interactive filtering and visualization belong to Phase 40.
- Retrieval-health aggregation belongs to Phase 41.
- Graph neighborhoods and communities belong to Phase 42.
- Agent previews belong to Phase 44.
</deferred>
