# Repo Context Packer

Repo Context Packer is a local-first, read-only context broker for coding agents.

The MVP exposes compact task context through:

- `AGENTS.md` for portable static instructions
- MCP tools/resources/prompts for dynamic context
- Thin native adapter files for Codex, Claude Code, Cursor, and OpenCode

## Development

```bash
cargo test --workspace
cargo run -p ctxpack -- --help
```

## Initialization

Initialize a repository with the portable AGENTS.md guidance and `.ctxpack/ctxpack.toml`:

```bash
cargo run -p ctxpack -- init --repo /path/to/repo
```

Generate optional native adapter files:

```bash
cargo run -p ctxpack -- init --repo /path/to/repo --cursor --claude --opencode
```

`ctxpack init` writes only repo-local files. It prints Codex MCP setup guidance but does not mutate global Codex configuration.

## MCP Runtime

Start the local stdio MCP server:

```bash
cargo run -p ctxpack -- serve-mcp
```

Implemented MCP tools:

- `prepare_task`
- `search`
- `related`
- `get_pack`
- `related_tests`
- `current_diff`

Implemented MCP resources include `ctxpack://repo/summary`, `ctxpack://repo/test-map`, `ctxpack://pack/guide`, safe file slices, and symbol search. Implemented prompts cover bugfix, feature, refactor, review, test-writing, and explanation workflows.

## Safe Inventory

Build the local file inventory for a repository:

```bash
cargo run -p ctxpack -- index --repo /path/to/repo
```

The inventory respects `.gitignore`, `.ctxpackignore`, and `.cursorignore`, excludes sensitive/generated files by default, and writes JSON under `~/.ctxpack/repos/<repo-id>/inventory.json`.

Generated and sensitive files require explicit opt-in:

```bash
cargo run -p ctxpack -- index --repo /path/to/repo --include-generated --include-sensitive
```

## Lexical Search

Search the safe inventory:

```bash
cargo run -p ctxpack -- search "requireSession" --repo /path/to/repo --limit 5
```

If no inventory exists for the repo, `ctxpack search` builds one using the safe default inventory rules before searching.

## Symbol Index

Extract language-aware symbols from safe inventoried files:

```bash
cargo run -p ctxpack -- symbols --repo /path/to/repo --limit 20
```

Search symbols by name, path, or signature:

```bash
cargo run -p ctxpack -- symbols --repo /path/to/repo --query requireSession --limit 5
```

The current local extractor covers TypeScript/JavaScript, Python, Rust, and Go definitions. MCP symbol resources use the same symbol search path through `ctxpack://symbol/<query>`.

## Related Tests

Find likely tests for changed source files:

```bash
cargo run -p ctxpack -- related-tests src/auth/session.ts --repo /path/to/repo
```

The result includes confidence, a reason, and a best-effort targeted test command.

## Git Co-Change Hints

Find files that have changed together in local git history:

```bash
cargo run -p ctxpack -- co-changes src/auth/session.ts --repo /path/to/repo --limit 5
```

Co-change hints read only local git metadata and are filtered through the safe inventory.

## Context Plan

Prepare a task-conditioned context plan:

```bash
cargo run -p ctxpack -- prepare-task "fix requireSession bug" --repo /path/to/repo --mode bug-fix
```

The plan fuses symbol search, lexical search, related tests, and local co-change hints into target files, line hints, validation commands, risk flags, and pack resource options.

## Context Pack

Materialize a budgeted context pack:

```bash
cargo run -p ctxpack -- get-pack "fix requireSession bug" --repo /path/to/repo --mode bug-fix --budget brief
```

Use `--format json` for structured output.

## Local Eval Traces

`prepare-task`, `get-pack`, and the matching MCP tools append source-free local traces under `~/.ctxpack/repos/<repo-id>/traces.jsonl`.

Inspect recent traces:

```bash
cargo run -p ctxpack -- eval traces --repo /path/to/repo --limit 20
```

Generate a manual dogfood checklist from recent traces:

```bash
cargo run -p ctxpack -- eval checklist --repo /path/to/repo --limit 5
```

Traces store task hashes, task type, target agent label, recommended files/tests/commands, optional pack id, optional budget, and created time. They do not store task text or source snippets.
