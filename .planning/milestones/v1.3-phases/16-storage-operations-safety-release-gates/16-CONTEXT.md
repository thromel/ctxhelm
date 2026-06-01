# Phase 16: Storage Operations, Safety, and Release Gates - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Mode:** Autonomous smart discuss

<domain>
## Phase Boundary

Phase 16 makes storage inspectable and recoverable through read-oriented operations, docs, and release-gate smoke coverage. Destructive behavior must require explicit confirmation.

</domain>

<decisions>
## Implementation Decisions

### Operations
- Add `storage init`, `status`, `repair`, `vacuum`, and `reset`.
- Make `reset` dry-run by default and require `--yes` for deletion.
- Surface compatibility, schema version, row counts, diagnostics, and database path.

### Safety
- Keep storage operations local-only and source-free.
- Do not run user project tests or mutate agent/global config.
- Add release-gate smoke coverage for repeated storage sync and source-body absence.

### the agent's Discretion
Use direct CLI output rather than adding MCP storage tools in this milestone; MCP diagnostics can consume the same typed reports later.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scripts/release-gate.sh` already has a selected-binary smoke-test structure.
- `scripts/check-release-docs.sh` already enforces documentation consistency.
- CLI command structure in `crates/ctxhelm/src/main.rs` supports nested subcommands.

### Established Patterns
- Release gates avoid publishing or mutating global agent config.
- Documentation should describe proof boundaries explicitly.

### Integration Points
- `docs/storage.md`
- `scripts/smoke-storage.sh`
- `scripts/release-gate.sh`

</code_context>

<specifics>
## Specific Ideas

Storage status should be understandable from the terminal without opening SQLite manually.

</specifics>

<deferred>
## Deferred Ideas

MCP storage diagnostics and automated migration repair policy can be deepened after CLI operations stabilize.

</deferred>
