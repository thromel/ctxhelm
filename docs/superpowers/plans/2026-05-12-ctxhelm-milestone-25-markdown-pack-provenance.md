# Milestone 25: Markdown Pack Provenance

## Goal

Make Markdown context packs carry the same source-free provenance as structured JSON packs.

## Scope

- Add repo id, task hash, and target agent to the Markdown pack header.
- Keep the fields source-free and stable for agent/eval inspection.
- Preserve existing pack sections, budgets, and privacy status.
- Update README wording so Markdown and JSON behavior are documented together.

## Non-goals

- Do not include task source beyond the existing task section.
- Do not change retrieval or pack budget allocation.
- Do not add new tools or resources.

## Verification

- Compiler Markdown rendering test for provenance fields.
- MCP `get_pack` Markdown test for target-agent/task-hash visibility.
- Full workspace tests, clippy, CLI help smoke, and live MCP smoke.
