# RepoWinnow Brand And Naming

RepoWinnow is the product name. `ctxpack` remains the CLI binary, Rust package
name, Homebrew formula, MCP server namespace, cache namespace, and compatibility
surface.

## Naming Model

| Surface | Name |
| --- | --- |
| Product and public brand | RepoWinnow |
| CLI command | `ctxpack` |
| Rust package and workspace crates | `ctxpack`, `ctxpack-core`, `ctxpack-index`, `ctxpack-compiler`, `ctxpack-mcp` |
| MCP resources and tools | `ctxpack://...`, `prepare_task`, `get_pack`, and related tool names |
| Descriptive category | repo context compiler or local context broker |

The preferred first mention is:

```text
RepoWinnow, powered by the ctxpack CLI
```

After that, use RepoWinnow when talking about the product and `ctxpack` when
talking about commands, packages, install paths, MCP resources, or file-system
state.

## Why This Name

RepoWinnow fits the product thesis: the value is not asking a repository
generic questions, but winnowing a noisy codebase down to a small,
task-conditioned evidence set that helps coding agents choose better files,
tests, and constraints.

It also avoids the most crowded naming patterns in this category:

- Do not use RepoLens as the product name. It already has direct public
  collisions across web, package, and repository surfaces.
- Do not use ContextMason as the product name. A separate adjacent MCP/code
  context product already uses Mason language, making the name too close for a
  serious public launch.
- Do not use bare Winnow as the product name. Existing LLM-context and AI tools
  use that word directly; the Repo prefix is part of the differentiator.
- Avoid names that imply autonomous editing, hosted code indexing, or replacing
  the user's existing agent.
- Avoid names that sound like generic code search, because the product is a
  context compiler.

This is a practical availability screen, not legal trademark clearance. Before a
large public launch, run a formal trademark and domain review.

## Messaging

Use:

- RepoWinnow builds the right repo context before agents edit.
- RepoWinnow is a local context compiler for AI coding agents.
- RepoWinnow makes Codex, Claude Code, Cursor, OpenCode, and similar agents
  inspect better files, tests, and constraints.

Avoid:

- RepoWinnow edits your code.
- RepoWinnow replaces your coding agent.
- RepoWinnow requires cloud embeddings or hosted indexing.
- RepoWinnow uploads repository source by default.
