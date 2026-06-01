# v2.0 Research Summary: Workspace & Team Layer

## Milestone Goal

Support multi-repo and team workflows while keeping source code local and agent-native surfaces primary.

## Stack Additions

- Typed workspace and team-policy contracts in `ctxhelm-core`.
- Local workspace metadata on top of existing per-repo storage.
- Source-free shared artifacts for cards, benchmark summaries, policy profiles, and privacy templates.
- Additive CLI/MCP changes instead of a large new product surface.

## Feature Table Stakes

- Multi-repo workspace manifest and status.
- Source-free workspace inventory aggregation.
- Task-to-repo routing before retrieval.
- Workspace-aware context plans and packs.
- Team-safe artifact export/import.
- Team privacy policy templates.
- Agent-native guidance for Codex, Claude Code, Cursor, and OpenCode.

## Architecture Direction

Keep per-repo stores authoritative. Add a thin workspace layer that routes tasks, aggregates source-free metadata, and preserves repo boundaries in all context plans and packs.

## Watch Outs

- Do not create hosted source indexing.
- Do not flatten repos into one noisy search space.
- Do not share raw prompts, source snippets, terminal logs, or model transcripts.
- Do not grow the MCP tool surface without a concrete workflow reason.
- Do not claim retrieval lift without fixed-sample evidence.

## Recommended Roadmap Shape

1. Workspace manifest and inventory aggregation.
2. Workspace-aware context planning.
3. Source-free shared artifacts and team privacy policy templates.
4. Agent-native docs, resources, smoke tests, and release gates.
