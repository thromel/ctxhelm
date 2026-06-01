# ctxhelm Milestone 13: Client Setup Artifacts

## Goal

Make the verified Claude Code MCP path reproducible from repo-local initialization artifacts without mutating a user's existing client configuration.

## Scope

- Keep `ctxhelm init` read-only with respect to global client configuration.
- Extend `ctxhelm init --claude` to generate a repo-local MCP config snippet under `.ctxhelm/adapters/`.
- Keep the Claude slash command as the behavior-level adapter.
- Document how the snippet relates to `.mcp.json`.
- Add tests that validate the generated MCP snippet and init report behavior.

## Verification

- focused `ctxhelm-core` tests for generated init artifacts
- full workspace test, clippy, and CLI help before closing the milestone
