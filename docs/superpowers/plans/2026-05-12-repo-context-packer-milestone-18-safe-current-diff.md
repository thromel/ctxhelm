# Milestone 18: Safe Current Diff

## Goal

Tighten the MCP `current_diff` tool so review workflows receive changed paths that respect ctxpack's safe inventory policy.

## Scope

- Filter unstaged, staged, and optionally untracked git path lists through safe inventory rules.
- Exclude sensitive, generated, ignored, deleted, or otherwise non-inventoried paths from returned path arrays.
- Return exclusion counts and privacy status instead of excluded path names or source content.
- Keep the MCP tool surface unchanged.

## Non-Goals

- No source diff content.
- No shell command execution beyond existing git path-list calls.
- No autonomous validation or editing.
- No cloud telemetry.

## Verification

- MCP unit test proves changed source paths are returned without source text.
- MCP unit test proves `.env` and generated paths are excluded and not leaked.
- Full workspace tests, clippy, and CLI help smoke.
