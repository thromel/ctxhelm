# Milestone 16: MCP Pack Resources

## Goal

Make `ContextPlan.packOptions[*].resourceUri` real for MCP clients. Agents should be able to call `prepare_task`, inspect the tiny plan, and then load the returned brief or standard pack URI only when they need more context.

## Scope

- Compile pack resources from the exact `ContextPlan` returned by `prepare_task`.
- Cache returned pack resources for the current MCP server session.
- Serve `ctxhelm://pack/<task-id>/<budget>` as Markdown.
- Serve `ctxhelm://pack/<task-id>/<budget>.json` as structured JSON.
- Keep the public MCP tool list unchanged.

## Non-Goals

- No cloud storage.
- No cross-session pack persistence.
- No autonomous file edits or command execution.
- No broad context injection by default.

## Verification

- Compiler test proves pack generation from an existing plan preserves the same task id.
- MCP test proves `prepare_task` makes returned pack URIs readable.
- Full workspace test suite.
- CLI help smoke.
