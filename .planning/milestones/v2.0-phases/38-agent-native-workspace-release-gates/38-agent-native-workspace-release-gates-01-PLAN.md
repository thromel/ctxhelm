# Phase 38 Plan: Agent-Native Workspace Release Gates

## Goal

Expose workspace behavior through agent-native surfaces and prove it in release
gates without expanding the MCP tool surface.

## Scope

- Keep the MCP tool list at the existing six tools.
- Add source-free workspace resources for status and shared artifact manifests.
- Document workspace-aware MCP resource usage for agents.
- Extend shared-artifacts smoke to exercise the new MCP resources through a
  live stdio server.
- Ensure release docs and packaging tests require the workspace/team smoke.

## Verification

- `cargo test -p ctxhelm-mcp resources`
- `cargo test -p ctxhelm script_contract --test release_packaging`
- `scripts/smoke-shared-artifacts.sh`
- `scripts/check-release-docs.sh`

