# Agent Setup Matrix

ctxpack is an agent-native, read-only context broker. It generates repo-local guidance and snippets that help existing agents call `prepare_task`, read files natively, and request `get_pack` only when they need more context.

## Support Matrix

| Agent | Generated artifact/snippet | Default write scope | Mutates global config by default | setup-check coverage | deterministic protocol proof | Optional real-client proof status | Verified-version/evidence notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Codex CLI | Managed `AGENTS.md` section plus printed MCP setup guidance | Repo-local guidance only; user copies any MCP config manually | No global config mutation | Verifies managed `AGENTS.md` and `.ctxpack/ctxpack.toml` | Supported through direct JSON-RPC/MCP smoke against `ctxpack serve-mcp` | Optional Codex smoke can require server-side `prepare_task` and `get_pack` request-log evidence, but current public proof records a source-free skip | Local command available: Codex CLI `0.44.0`; latest public archive smoke did not produce machine-checkable tool-call evidence, so Codex remains optional evidence only |
| Claude Code | `.claude/commands/ctxpack-bugfix.md` and `.ctxpack/adapters/claude-mcp.json` | Repo/project-local command and mergeable MCP snippet | No global config mutation; user merges project MCP config when desired | Verifies Claude command and adapter snippet when `--claude` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxpack serve-mcp` | Optional Claude smoke and workflow eval can require server-side `prepare_task` and `get_pack` request-log evidence | Local command available: Claude Code `2.1.159`; latest workflow evidence used strict MCP config, explicit `repo`, `prepare_task`, and `get_pack` |
| Cursor | `.cursor/rules/ctxpack.mdc` | Repo-local rule file | No global config mutation | Verifies Cursor rule when `--cursor` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxpack serve-mcp` | Not claimed as machine-checkable tool-call evidence in v1.1 | Local command available: Cursor `3.3.30`; version presence is not tool-call proof |
| OpenCode | `.ctxpack/adapters/opencode.jsonc.snippet` | Repo-local snippet for manual merge | No global config mutation | Verifies OpenCode snippet when `--opencode` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxpack serve-mcp` | Not claimed as machine-checkable tool-call evidence in v1.1 | Local command available: OpenCode `1.14.25`; version presence is not tool-call proof |

## Proof Taxonomy

**Generated artifact checks** are read-only setup validation. `ctxpack setup-check --repo "$REPO" --cursor --claude --opencode` confirms expected repo-local files exist and contain recognizable ctxpack guidance. It does not run real agent clients.

**deterministic protocol proof** means direct JSON-RPC/MCP smoke through `ctxpack serve-mcp`. The smoke sends machine-checkable tool calls with an explicit `repo`, verifies `prepare_task` returns target files and pack options, verifies `get_pack` returns a structured pack, reads context-area resources from a non-repo server cwd, and can read a pack resource during the same MCP server session.

**real-client proof** means an actual agent client starts ctxpack as an MCP server and produces request-log evidence for tool calls. For v1.1, this optional proof path is maintained for Codex CLI and Claude Code because the smoke scripts can inspect server-side requests for explicit-repo `prepare_task` and `get_pack` calls tied to exact client versions.

For Claude Code, maintainers can run the deeper workflow eval:

```bash
CTXPACK_RUN_REAL_CLIENT=1 bash scripts/e2e-claude-workflow.sh
```

That wrapper records a source-free workflow report proving Claude made the
expected explicit-repo `prepare_task` and `get_pack` calls through MCP. It keeps
only hashes and sanitized request summaries, not raw prompts, raw MCP traffic,
source text, or user-project command output.

Latest local real-client workflow refresh: 2026-06-01, Claude Code `2.1.159`
passed `scripts/e2e-claude-workflow.sh` with source-free explicit-repo
`prepare_task` and `get_pack` evidence. See
`.planning/e2e/2026-06-01-phase132-claude-workflow-eval.md`. Codex CLI
`0.44.0` remains optional evidence only because the latest public archive smoke
recorded a source-free skip instead of machine-checkable tool-call evidence.

Cursor and OpenCode setup can be checked through generated artifacts plus deterministic protocol proof, including source-free context-area resource reads. v1.1 does not claim machine-checkable Cursor tool-call proof, and it does not claim machine-checkable OpenCode tool-call proof.

The setup-proof wrappers are:

```bash
bash scripts/smoke-cursor-mcp.sh
bash scripts/smoke-opencode-mcp.sh
```

They generate repo-local adapter files in a temporary repo, run `setup-check`,
run the deterministic MCP protocol smoke, and optionally write JSON evidence to
`CTXPACK_REAL_CLIENT_EVIDENCE_DIR`. The evidence explicitly marks
`realClientToolCalls: false` so setup proof is not confused with tool-call
transcript proof.

## Common Agent Flow

Use the same sequence across clients:

1. Call `prepare_task` with explicit `repo`, task text, mode, and active paths when known.
2. Let the agent use native file reads for the top targets and tests.
3. Call `get_pack` with the same explicit `repo` when direct file reads or brief plan context are insufficient.

Example MCP tool arguments:

```json
{
  "task": "fix requireSession bug",
  "repo": "/path/to/repo",
  "mode": "bug_fix",
  "paths": ["src/auth/session.ts"],
  "targetAgent": "codex",
  "recordTrace": false
}
```

For a durable reconnect path, call `get_pack` directly:

```json
{
  "task": "fix requireSession bug",
  "repo": "/path/to/repo",
  "mode": "bug_fix",
  "budget": "brief",
  "format": "json",
  "paths": ["src/auth/session.ts"],
  "recordTrace": false
}
```

Pack resource URIs returned by `prepare_task` are available during the same MCP server session. After reconnecting, ask for `get_pack` instead of reading an old URI.

## Codex CLI Notes

Run:

```bash
ctxpack init --repo "$REPO"
ctxpack doctor --repo "$REPO"
ctxpack setup-check --repo "$REPO"
```

Codex setup is copy/paste-oriented. ctxpack does not edit `CODEX_HOME`, user config, or global MCP settings. Use an absolute `ctxpack` binary path in Codex MCP config when GUI or shell `PATH` differs from your terminal.

## Claude Code Notes

Run:

```bash
ctxpack init --repo "$REPO" --claude
ctxpack doctor --repo "$REPO"
ctxpack setup-check --repo "$REPO" --claude
```

The Claude path writes repo-local command guidance and a mergeable `.ctxpack/adapters/claude-mcp.json` snippet. ctxpack does not automatically write `.mcp.json`; the user or project maintainer chooses whether to merge the snippet.

## Cursor Notes

Run:

```bash
ctxpack init --repo "$REPO" --cursor
ctxpack doctor --repo "$REPO"
ctxpack setup-check --repo "$REPO" --cursor
```

The Cursor path writes `.cursor/rules/ctxpack.mdc`. That rule explains how to call ctxpack through configured MCP surfaces, but setup validation only checks the generated file. It is not real-client request evidence.

## OpenCode Notes

Run:

```bash
ctxpack init --repo "$REPO" --opencode
ctxpack doctor --repo "$REPO"
ctxpack setup-check --repo "$REPO" --opencode
```

The OpenCode path writes `.ctxpack/adapters/opencode.jsonc.snippet`. It is a repo-local snippet for manual review or merge and is not applied to a global OpenCode configuration by ctxpack.
