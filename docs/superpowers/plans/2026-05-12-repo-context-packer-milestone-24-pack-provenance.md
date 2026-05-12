# Milestone 24: Pack Provenance

## Goal

Align `ContextPack` with the product contract by adding source-free provenance fields agents and eval tooling can rely on.

## Scope

- Add `repoId`, `taskHash`, and `targetAgent` to serialized `ContextPack`.
- Keep task text and source text out of structured provenance.
- Propagate CLI and MCP `targetAgent` into generated packs.
- Preserve existing compile helpers with `generic` defaults for callers that do not specify an agent.

## Non-goals

- Do not change pack section content or budget allocation.
- Do not add cloud telemetry or remote upload.
- Do not change eval trace storage.

## Verification

- Contract serialization test for the new fields.
- Compiler tests for generic and explicit target-agent pack provenance.
- MCP `get_pack` test for `targetAgent`, `repoId`, and `taskHash`.
- Full workspace tests, clippy, CLI help smoke, and live MCP smoke.
