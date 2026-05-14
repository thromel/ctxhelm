# Phase 14: Incremental Indexing & Cache Rebuilds - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Mode:** Autonomous smart discuss

<domain>
## Phase Boundary

Phase 14 makes the new SQLite store useful for repeated indexing. The phase should not replace every existing JSON fallback yet; it should add source-free storage synchronization and diagnostics that prove unchanged safe file records are reused while changed/deleted/new records are updated.

</domain>

<decisions>
## Implementation Decisions

### Storage Sync Boundary
- Keep JSON inventory behavior intact for compatibility.
- Add storage sync as an explicit `index --store` path instead of silently changing the default CLI behavior.
- Compare path, language, role, hash, size, generated/ignored status, and policy-derived inventory counts.
- Report source-free reuse/update/create/delete counts.

### Privacy
- Store hashes, safe paths, roles, sizes, and counts only.
- Do not persist raw source, snippets, prompt text, secrets, or file bodies.
- Preserve generated and sensitive exclusion counts as diagnostics, not content.

### the agent's Discretion
Implementation details can stay in `ctxpack-index::storage` as long as the crate facade exposes typed reports for CLI and future compiler consumers.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ctxpack-index/src/inventory.rs` already builds safe inventory entries and manifest metadata.
- `crates/ctxpack-index/src/storage.rs` already initializes the source-free SQLite schema.
- `crates/ctxpack/src/main.rs` owns CLI rendering and can expose storage sync without changing MCP behavior.

### Established Patterns
- Public CLI output is compact Markdown/text by default, with JSON formats where already supported.
- Source-free diagnostics use typed `Diagnostic` records and policy-filtered paths.

### Integration Points
- `ctxpack index --repo <path>` is the natural command to attach storage-backed incremental sync.
- `StoreConfig` remains the typed entry point for default or explicit database paths.

</code_context>

<specifics>
## Specific Ideas

Use a fresh temp clone/worktree for implementation because the original Documents checkout had iCloud dataless files that blocked reads.

</specifics>

<deferred>
## Deferred Ideas

Storage-backed symbol/test/dependency reuse can deepen later; Phase 14 must at least provide durable file-record reuse and source-free counts.

</deferred>
