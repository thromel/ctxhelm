# ctxhelm Milestone 2: MCP Skeleton

**Goal:** Replace the `serve-mcp` print-only stub with a small stdio MCP server that coding agents can discover and call for `prepare_task`.

This milestone is intentionally narrower than the full MCP runtime. It proves the agent-native path before safe inventory, lexical search, resources, prompts, or context packs exist.

## Scope

- Implement newline-delimited JSON-RPC over stdio.
- Handle `initialize`.
- Handle `notifications/initialized` as a no-op notification.
- Handle `tools/list` with deterministic tool ordering.
- Handle `tools/call` for `prepare_task`.
- Return typed `ContextPlan` JSON as both structured content and a text block for compatibility.
- Keep the other planned tool names visible only if their behavior is honest and explicitly not implemented, or omit them until they work.
- Keep all behavior read-only.

## Out of Scope

- MCP resources.
- MCP prompts.
- Search/index behavior.
- Tool calls that read source files.
- Global agent configuration mutation.
- Cloud calls.

## Acceptance Checks

- `ctxhelm serve-mcp` accepts `initialize`, `tools/list`, and `tools/call` for `prepare_task` on stdin.
- `tools/list` exposes a small deterministic tool list.
- `prepare_task` validates required arguments and returns a `ContextPlan`.
- Unknown methods return JSON-RPC method-not-found errors.
- `cargo test --workspace --locked` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
