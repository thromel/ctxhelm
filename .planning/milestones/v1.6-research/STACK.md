# v1.6 Research: Stack

## Scope

This research covers only the new v1.6 surface: durable repo memory and experience cards that are source-free, local-first, and selectively retrieved by ctxhelm.

## Findings

- Reuse the existing Rust workspace and SQLite storage foundation. v1.3 already created source-free storage for files, semantic vectors, packs, evals, proofs, and traces; v1.6 should add memory/card tables instead of introducing another store.
- Keep generated memory as Markdown plus source-free metadata. Claude Code project memory uses `CLAUDE.md` and imports, Cursor rules are repo-local `.cursor/rules` files, and AGENTS.md is a broad cross-agent static surface. ctxhelm should not duplicate those as the source of truth; it should generate small source-linked cards and expose them through CLI/MCP.
- Use MCP tools/resources for dynamic memory selection. MCP tools are model-controlled, and resources fit larger context objects. ctxhelm already exposes both, so v1.6 should add memory/cards behind existing `prepare_task`, `get_pack`, and resources rather than adding many new tools.
- Keep embeddings optional. v1.4 semantic retrieval is local-only and explicit; memory selection can start with lexical, card metadata, source links, graph/test relations, and task type before using semantic signals.
- Prefer deterministic extractive summaries first. The current `cards generate` path is deterministic and source-snippet-free. v1.6 should extend this with freshness metadata, regeneration triggers, and human-editable cards before adding model-authored prose.

## Sources

- Claude Code memory docs: https://docs.claude.com/en/docs/claude-code/memory
- Cursor rules docs: https://docs.cursor.com/context/rules
- AGENTS.md project: https://github.com/openai/agents.md
- MCP tools docs: https://modelcontextprotocol.io/docs/concepts/tools
- Aider repo map docs: https://aider.chat/docs/repomap.html
