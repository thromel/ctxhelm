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
`--claude` writes a slash-command file plus `.ctxpack/adapters/claude-mcp.json`, a project MCP config snippet you can copy or merge into `.mcp.json`.

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

`current_diff` returns safe changed path lists only. Paths excluded by ignore, generated, sensitive, or other safe-inventory policy are summarized by count and source text is never returned.

Implemented MCP resources include `ctxpack://repo/summary`, package-aware `ctxpack://repo/test-map`, `ctxpack://repo/dependency-graph`, `ctxpack://pack/guide`, session-scoped `ctxpack://pack/<task-id>/<budget>` resources returned by `prepare_task`, safe file slices, and symbol search. Implemented prompts cover bugfix, feature, refactor, review, test-writing, and explanation workflows.

## Client Integration Status

Current local smoke status:

- Claude Code `2.1.92`: end-to-end MCP tool use verified with `.mcp.json`, `--strict-mcp-config`, and an explicit `repo` argument. Claude connected to ctxpack, called `prepare_task`, and received target files plus `pnpm vitest run <test>` validation.
- Codex CLI `0.130.0`: end-to-end MCP `prepare_task` use verified with a temporary `CODEX_HOME`, configured ctxpack server, explicit `repo` argument, and non-interactive approval bypass for the smoke run.

When using ctxpack through MCP, pass the active repository path as `repo` whenever the client knows it. Some clients launch MCP servers from a different working directory than the project they expose.

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
For JavaScript and TypeScript repos, ctxpack now checks nearby `package.json` scripts and package-manager lockfiles to prefer commands such as `pnpm vitest run <test>` or `npm test -- <test>` instead of assuming a single runner.

The MCP `ctxpack://repo/test-map` resource uses the same package-aware command inference for safe inventoried test files.

## Git Co-Change Hints

Find files that have changed together in local git history:

```bash
cargo run -p ctxpack -- co-changes src/auth/session.ts --repo /path/to/repo --limit 5
```

Co-change hints read only local git metadata and are filtered through the safe inventory.

## Dependency Graph

Inspect safe local import edges around a file:

```bash
cargo run -p ctxpack -- dependencies src/auth/session.ts --repo /path/to/repo --limit 10
```

Return the current safe dependency graph:

```bash
cargo run -p ctxpack -- dependencies --all --repo /path/to/repo --limit 50
```

Dependency edges are inferred from local TypeScript/JavaScript, Python, and Rust imports in safe source/test files. External packages, generated files, sensitive files, and ignored files are excluded by default. MCP clients can request dependency expansion through `related` with `include: ["dependencies"]`, and can read the repository graph resource at `ctxpack://repo/dependency-graph`.

## Context Plan

Prepare a task-conditioned context plan:

```bash
cargo run -p ctxpack -- prepare-task "fix requireSession bug" --repo /path/to/repo --mode bug-fix
```

Pass active editor files as repeatable anchors when the host agent knows them:

```bash
cargo run -p ctxpack -- prepare-task "fix redirect behavior" --repo /path/to/repo --mode bug-fix --path src/auth/middleware.ts
```

The plan fuses active path anchors, symbol search, lexical search, related tests, local dependency edges, and local co-change hints into target files, line hints, validation commands, risk flags, and pack resource options. MCP clients can pass the same active/open files through the `paths` array on `prepare_task`.

For MCP clients, the `packOptions[*].resourceUri` values returned by `prepare_task` are loadable during the same MCP server session. Add `.json` to a returned pack URI to read the structured pack resource instead of Markdown.

## Context Pack

Materialize a budgeted context pack:

```bash
cargo run -p ctxpack -- get-pack "fix requireSession bug" --repo /path/to/repo --mode bug-fix --budget brief
```

Use `--format json` for structured output. `get-pack` also accepts repeatable `--path <file>` anchors, and the MCP `get_pack` tool accepts the same `paths` array.

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
