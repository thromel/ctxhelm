# v1.6 Research: Features

## Table Stakes

- Domain cards for important subsystems with stable IDs, source links, input hashes, freshness state, and regeneration reasons.
- Experience cards derived from prior local traces and explicit user corrections, without storing prompt text or source snippets.
- Memory retrieval as a bounded signal inside `prepare_task` and `get_pack`, with explicit provenance and confidence.
- Human review controls: list pending/stale cards, approve or disable cards, redact unsafe generated text, and regenerate cards safely.
- MCP/resource exposure for selected memory so agents can load memory progressively instead of receiving every card.

## Differentiators

- Source-free card metadata and source-linked evidence rather than opaque chat memory.
- Budgeted memory selection, so memory competes with lexical/vector/graph/test/history context instead of bloating packs.
- Freshness-aware generated cards, so stale cards degrade with diagnostics instead of misleading agents.
- Experience cards grounded in actual agent sessions, test failures, accepted fixes, and user corrections.
- Release/eval proof that memory helps retrieval or pack quality without increasing irrelevant context.

## Anti-Features

- Do not auto-write arbitrary lessons from raw chat transcripts.
- Do not make card generation cloud-dependent.
- Do not inject all memory into AGENTS.md or every pack.
- Do not make ctxpack an autonomous session logger that captures private prompts by default.
- Do not claim full learning/tuning; adaptive policy is v1.7.

## Sources

- Claude Code memory docs: https://docs.claude.com/en/docs/claude-code/memory
- OpenHands microagents docs: https://docs.all-hands.dev/usage/prompting/microagents-overview
- Continue context providers: https://docs.continue.dev/customize/custom-providers
- Cursor rules docs: https://docs.cursor.com/context/rules

