# v1.6 Research: Architecture

## Existing Integration Points

- `crates/ctxhelm-compiler/src/cards.rs` already generates source-free repo, testing, and dependency cards.
- `crates/ctxhelm-index/src/storage.rs` already owns source-free SQLite schema/migrations and can be extended for memory metadata.
- `crates/ctxhelm-index/src/traces.rs` already stores source-free local eval traces.
- `crates/ctxhelm-compiler/src/planning.rs` already fuses retrieval signals into `ContextPlan`.
- `crates/ctxhelm-compiler/src/packs.rs` already compiles budgeted packs from plans.
- `crates/ctxhelm-mcp/src/lib.rs` already exposes tools/resources/prompts and session-scoped pack resources.

## Proposed v1.6 Components

1. Memory contracts:
   - `MemoryCard`
   - `MemoryCardKind`
   - `MemoryFreshness`
   - `MemoryCandidate`
   - `MemorySelection`
   - source-free JSON shapes for CLI/MCP.

2. Storage extension:
   - `memory_cards` table with card ID, kind, title, summary, source links, input hashes, status, privacy label, and timestamps.
   - `experience_events` or source-free derivation records from traces and explicit corrections.
   - no raw prompt text, source snippets, terminal logs, or secrets.

3. Card generation:
   - extend current `cards generate` or add `memory generate` around the same safe inventory policy.
   - generate subsystem/domain cards from packages, dependency clusters, tests, symbols, and docs.
   - generate experience cards from accepted source-free events.

4. Selection:
   - retrieve memory candidates during `prepare_task` and `get_pack`.
   - rank by lexical/task overlap, source-link proximity to target files, card kind, freshness, prior usefulness, and budget.
   - cap selected memory separately from source/test snippets.

5. Review:
   - CLI operations for list/show/approve/disable/regenerate.
   - diagnostics for stale, disabled, unreviewed, unsafe, or oversized cards.
   - docs and release smokes for source-free persistence and pack integration.

## Build Order

1. Contracts and storage schema.
2. Freshness-aware domain card generation.
3. Experience card ingestion from source-free traces/corrections.
4. Memory candidate ranking and pack integration.
5. Review workflow, docs, tests, and release gate.

## Sources

- Aider repo map docs: https://aider.chat/docs/repomap.html
- MCP tools docs: https://modelcontextprotocol.io/docs/concepts/tools
- Claude Code memory docs: https://docs.claude.com/en/docs/claude-code/memory
- Cursor rules docs: https://docs.cursor.com/context/rules

