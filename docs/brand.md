# ctxhelm Brand And Naming

ctxhelm is the product name. `ctxhelm` is also the CLI binary, Rust package
name, Homebrew formula, MCP server namespace, cache namespace, and release
surface.

## Naming Model

| Surface | Name |
| --- | --- |
| Product and public brand | ctxhelm |
| CLI command | `ctxhelm` |
| Rust package and workspace crates | `ctxhelm`, `ctxhelm-core`, `ctxhelm-index`, `ctxhelm-compiler`, `ctxhelm-mcp` |
| MCP resources and tools | `ctxhelm://...`, `prepare_task`, `get_pack`, and related tool names |
| Descriptive category | repo context compiler or local context broker |

The preferred first mention is:

```text
ctxhelm, the local context compiler for coding agents
```

After that, use ctxhelm for the product and `ctxhelm` for commands, packages,
install paths, MCP resources, or file-system state.

## Why This Name

ctxhelm fits the product thesis: the tool acts like a helm for coding agents.
It does not replace the agent or take over editing; it steers the agent toward
the files, tests, graph edges, history, constraints, and source-free memory that
matter for the current task.

It also avoids the most crowded naming patterns in this category:

- Do not use generic repo-search or context-search names as the product name.
  They are crowded, weakly differentiating, and easy to confuse with adjacent
  tools.
- Do not reuse rejected candidate names in public copy, command examples,
  package names, generated adapters, or documentation.
- Avoid names that imply autonomous editing, hosted code indexing, or replacing
  the user's existing agent.
- Avoid names that sound like generic code search, because the product is a
  context compiler.

This is a practical availability screen, not legal trademark clearance. Before a
large public launch, run a formal trademark and domain review.

## Messaging

Use:

- ctxhelm builds the right repo context before agents edit.
- ctxhelm is a local context compiler for AI coding agents.
- ctxhelm makes Codex, Claude Code, Cursor, OpenCode, and similar agents
  inspect better files, tests, and constraints.

Avoid:

- ctxhelm edits your code.
- ctxhelm replaces your coding agent.
- ctxhelm requires cloud embeddings or hosted indexing.
- ctxhelm uploads repository source by default.
