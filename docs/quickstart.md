# First-Pack Quickstart

This guide starts from an installed `ctxpack` binary and an existing git repository. It uses explicit `--repo` arguments throughout because agent clients and MCP servers often start from a different working directory than the repository you are editing.

## Prerequisites

- `ctxpack` installed on `PATH`, or an absolute path to the binary for MCP client configuration.
- A local git repository you want to inspect.
- A task that can benefit from file, test, dependency, and pack guidance.

ctxpack is local-first and read-only. It does not edit source files, run your project tests, install dependencies, or mutate global agent configuration.

## Verify The Install

```bash
ctxpack --version
ctxpack --help
```

For v1.1.0, `ctxpack --version` should print `ctxpack 1.1.0`. If the command is not found, fix your shell or agent `PATH`, or use an absolute binary path in the MCP configuration.

## Choose A Repo

```bash
export REPO=/path/to/repo
```

Use an absolute path when possible. The same explicit `--repo` value should appear in CLI commands and MCP tool arguments.

## Initialize Repo-Local Guidance

```bash
ctxpack init --repo "$REPO" --cursor --claude --opencode
```

This writes repo-local guidance and optional adapter snippets:

- `AGENTS.md` managed ctxpack section
- `.ctxpack/ctxpack.toml`
- `.cursor/rules/ctxpack.mdc` when `--cursor` is used
- `.claude/commands/ctxpack-bugfix.md` and `.ctxpack/adapters/claude-mcp.json` when `--claude` is used
- `.ctxpack/adapters/opencode.jsonc.snippet` when `--opencode` is used

Codex setup remains copy/paste-oriented. ctxpack prints guidance but does not mutate global Codex configuration.

## Validate Setup

```bash
ctxpack setup-check --repo "$REPO" --cursor --claude --opencode
```

`setup-check` validates repo-local generated artifacts. It does not run real agent clients, edit client configuration, or prove that Cursor, Claude Code, Codex CLI, or OpenCode called a tool.

## Deterministic MCP Proof Context

The hard automated proof for v1.1 is deterministic JSON-RPC/MCP protocol smoke through `ctxpack serve-mcp`. That proof starts the ctxpack MCP server, sends machine-checkable `prepare_task` and `get_pack` calls with an explicit `repo`, and inspects structured responses.

Real-client proof is separate and optional. Codex CLI and Claude Code smokes can be tied to exact local client versions and request logs. Cursor and OpenCode setup is validated through generated artifact checks plus deterministic protocol proof; v1.1 docs do not claim machine-checkable tool-call proof for those two clients.

## First Prepare Task

Ask for a task-conditioned plan:

```bash
ctxpack prepare-task "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --path src/auth/session.ts
```

Use `--path` for active editor files or known anchors. The plan combines safe inventory, symbols, lexical matches, related tests, dependency edges, co-change hints, and current-diff anchors when requested.

The response includes target files, related tests, validation commands, risk flags, diagnostics, and `packOptions`. Each pack option includes a budget and, for MCP flows, a resource URI.

## First Get Pack

Materialize a compact pack with `--budget brief`:

```bash
ctxpack get-pack "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --budget brief \
  --path src/auth/session.ts
```

Use JSON output when another tool needs structured data:

```bash
ctxpack get-pack "fix requireSession bug" \
  --repo "$REPO" \
  --mode bug-fix \
  --budget brief \
  --path src/auth/session.ts \
  --format json
```

Brief packs are intended for a small first context handoff. Standard and deep packs include more material when the agent needs broader context.

## Pack Options And Session Scope

`prepare_task` returns pack resource URIs for MCP clients. Those resource URIs are session-scoped: they are available during the same MCP server session that produced them. After a reconnect or server restart, call `get_pack` with the same task, repo, mode, budget, and paths instead of relying on an old resource URI.

In an agent workflow, a good default sequence is:

1. Call `prepare_task` with an explicit `repo`.
2. Let the agent use native file reads for the top targets.
3. Call `get_pack` progressively when direct file reads or brief plan context are not enough.

## Maintainer Source-Checkout Validation

Maintainers working from a source checkout can run the first-pack smoke script against an installed or locally built binary:

```bash
CTXPACK_BIN=/absolute/path/to/ctxpack bash scripts/smoke-first-pack.sh
```

This is a source-checkout validation script, not the normal user setup path.
