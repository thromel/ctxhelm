# ContextMason Brand And Naming

ContextMason is the product name. `ctxpack` remains the CLI binary, Rust package
name, Homebrew formula, MCP server namespace, cache namespace, and compatibility
surface.

## Naming Model

| Surface | Name |
| --- | --- |
| Product and public brand | ContextMason |
| CLI command | `ctxpack` |
| Rust package and workspace crates | `ctxpack`, `ctxpack-core`, `ctxpack-index`, `ctxpack-compiler`, `ctxpack-mcp` |
| MCP resources and tools | `ctxpack://...`, `prepare_task`, `get_pack`, and related tool names |
| Descriptive category | repo context compiler or local context broker |

The preferred first mention is:

```text
ContextMason, powered by the ctxpack CLI
```

After that, use ContextMason when talking about the product and `ctxpack` when
talking about commands, packages, install paths, MCP resources, or file-system
state.

## Why This Name

ContextMason fits the product thesis: the value is not asking a repository
generic questions, but carefully constructing a small, task-conditioned evidence
set that helps coding agents choose better files, tests, and constraints.

It also avoids the most crowded naming patterns in this category:

- Do not use RepoLens as the product name. It already has direct public
  collisions across web, package, and repository surfaces.
- Avoid names that imply autonomous editing, hosted code indexing, or replacing
  the user's existing agent.
- Avoid names that sound like generic code search, because the product is a
  context compiler.

This is a practical availability screen, not legal trademark clearance. Before a
large public launch, run a formal trademark and domain review.

## Messaging

Use:

- ContextMason builds the right repo context before agents edit.
- ContextMason is a local context compiler for AI coding agents.
- ContextMason makes Codex, Claude Code, Cursor, OpenCode, and similar agents
  inspect better files, tests, and constraints.

Avoid:

- ContextMason edits your code.
- ContextMason replaces your coding agent.
- ContextMason requires cloud embeddings or hosted indexing.
- ContextMason uploads repository source by default.
