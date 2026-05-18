# v1.6 Research: Pitfalls

## Pitfalls

- Memory bloat: if every card is injected into every pack, ctxpack becomes the context dump it was designed to avoid.
- Stale generated summaries: architecture cards become harmful when source files move or behavior changes.
- Privacy drift: experience cards can accidentally store prompt text, terminal logs, or source snippets if derived too eagerly.
- Unreviewed model prose: generated summaries can sound authoritative while being wrong or obsolete.
- Tool surface sprawl: adding many MCP tools for memory would increase agent decision overhead.
- Evaluation ambiguity: if memory changes retrieval and pack output without metrics, we cannot prove it helps.

## Prevention Strategy

- Add strict source-free storage tests before card selection.
- Track source links and input hashes for each card.
- Default to reviewed or deterministic cards in packs; mark unreviewed cards separately.
- Cap memory tokens and expose memory as one retrieval signal with evidence.
- Keep MCP additions small, ideally resources and additive fields on existing tools.
- Add release smoke that proves memory data is local-only, source-free, freshness-aware, and budgeted.

## Sources

- Claude Code memory docs: https://docs.claude.com/en/docs/claude-code/memory
- Cursor rules docs: https://docs.cursor.com/context/rules
- Continue context providers: https://docs.continue.dev/customize/custom-providers
- MCP tools docs: https://modelcontextprotocol.io/docs/concepts/tools

