# v1.6 Research Summary

## Stack Additions

- Extend existing Rust crates and SQLite storage; do not add a new service or cloud dependency.
- Add source-free memory/card contracts and storage tables.
- Reuse current safe inventory, cards, traces, planning, packs, and MCP resource surfaces.

## Feature Table Stakes

- Freshness-aware domain cards.
- Source-free experience cards from traces, test failures, accepted fixes, and user corrections.
- Memory selection in `prepare_task` and `get_pack` with budget caps and evidence.
- Human review, disable, redact, and regenerate controls.
- Release/eval proof that memory is local-only and useful.

## Watch Out For

- Do not inject all memory by default.
- Do not store raw source, prompts, terminal logs, or secrets in memory.
- Do not trust stale generated summaries.
- Do not add a large MCP tool surface.
- Do not call this adaptive learning; v1.7 owns policy tuning.

## Recommended v1.6 Shape

Five phases:

1. Memory contracts and storage schema.
2. Freshness-aware domain cards.
3. Source-free experience cards.
4. Memory selection in plans and packs.
5. Review workflow, docs, release gates, and eval proof.

## Sources

- Claude Code memory docs: https://docs.claude.com/en/docs/claude-code/memory
- Cursor rules docs: https://docs.cursor.com/context/rules
- AGENTS.md project: https://github.com/openai/agents.md
- MCP tools docs: https://modelcontextprotocol.io/docs/concepts/tools
- Aider repo map docs: https://aider.chat/docs/repomap.html
- Continue context providers: https://docs.continue.dev/customize/custom-providers
