# Agent Setup Matrix

ctxhelm is an agent-native, read-only context broker. It generates repo-local guidance and snippets that help existing agents call `prepare_task`, read returned target files natively, and request `get_pack` only when they need more context. Treat path discovery as a weaker signal than file consumption: agents should read the files they rely on before editing or answering.

## Support Matrix

| Agent | Generated artifact/snippet | Default write scope | Mutates global config by default | setup-check coverage | deterministic protocol proof | Optional real-client proof status | Verified-version/evidence notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Codex CLI | Managed `AGENTS.md` section plus printed MCP setup guidance | Repo-local guidance only; user copies any MCP config manually | No global config mutation | Verifies managed `AGENTS.md` and `.ctxhelm/ctxhelm.toml` | Supported through direct JSON-RPC/MCP smoke against `ctxhelm serve-mcp` | Optional Codex smoke can require server-side `prepare_task` and `get_pack` request-log evidence; current local outcome proof passes with explicit-repo tool-call evidence | Local command available: Codex CLI `0.137.0`; current local matrix records five source-free lanes, four comparable ctxhelm lanes, no forbidden commands, and outcome claim `ctxhelm_improved` |
| Claude Code | `.claude/commands/ctxhelm-bugfix.md` and `.ctxhelm/adapters/claude-mcp.json` | Repo/project-local command and mergeable MCP snippet | No global config mutation; user merges project MCP config when desired | Verifies Claude command and adapter snippet when `--claude` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxhelm serve-mcp` | Optional Claude smoke and workflow eval can require server-side `prepare_task` and `get_pack` request-log evidence; current availability preflight is rate-limited | Local command available: Claude Code `2.1.163`; current availability proof classifies API status `429` separately from ctxhelm protocol and retrieval behavior |
| Cursor | `.cursor/rules/ctxhelm.mdc` | Repo-local rule file | No global config mutation | Verifies Cursor rule when `--cursor` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxhelm serve-mcp` | Optional Cursor Agent CLI smoke can require server-side `prepare_task` and `get_pack` request-log evidence; current local proof is skipped because Cursor Agent is not logged in | Local command available: Cursor `3.6.21`; version presence alone is not tool-call proof |
| OpenCode | `.ctxhelm/adapters/opencode.jsonc.snippet` | Repo-local snippet for manual merge | No global config mutation | Verifies OpenCode snippet when `--opencode` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxhelm serve-mcp` | Optional OpenCode smoke passes locally with server-side `prepare_task` and `get_pack` request-log evidence | Local command available: OpenCode `1.14.25`; current proof records two explicit-repo MCP tool calls |

## Proof Taxonomy

**Generated artifact checks** are read-only setup validation. `ctxhelm setup-check --repo "$REPO" --cursor --claude --opencode` confirms expected repo-local files exist and contain recognizable ctxhelm guidance. It does not run real agent clients.

**deterministic protocol proof** means direct JSON-RPC/MCP smoke through `ctxhelm serve-mcp`. The smoke sends machine-checkable tool calls with an explicit `repo`, verifies `prepare_task` returns target files and pack options, verifies `get_pack` returns a structured pack, reads context-area resources from a non-repo server cwd, and can read a pack resource during the same MCP server session.

**real-client proof** means an actual agent client starts ctxhelm as an MCP server and produces request-log evidence for tool calls. For v1.1, this optional proof path is maintained for Codex CLI, Claude Code, Cursor Agent CLI, and OpenCode because the smoke scripts can inspect server-side requests for explicit-repo `prepare_task` and `get_pack` calls tied to exact client versions.

For Claude Code, maintainers can run the deeper workflow eval:

```bash
CTXHELM_RUN_REAL_CLIENT=1 bash scripts/e2e-claude-workflow.sh
```

That wrapper records a source-free workflow report proving Claude made the
expected explicit-repo `prepare_task` and `get_pack` calls through MCP. It keeps
only hashes and sanitized request summaries, not raw prompts, raw MCP traffic,
source text, or user-project command output.

For process-level comparison against native Claude exploration, maintainers can
run the paired agent-run eval:

```bash
CTXHELM_RUN_REAL_CLIENT=1 bash scripts/e2e-agent-run.sh \
  --repo "$REPO" \
  --task "Identify the files relevant to the requested change" \
  --target-file path/to/expected/file \
  --output .ctxhelm/e2e/agent-run-claude.json
ctxhelm eval agent-run --report .ctxhelm/e2e/agent-run-claude.json
```

That wrapper runs native baseline, `prepare_task`, brief-pack, standard-pack,
and memory-guided standard-pack Claude lanes. It reports target coverage,
target-read coverage, read-file count, irrelevant reads, tool-call count,
required ctxhelm call compliance, and observed ctxhelm tool calls while keeping
raw prompts, raw stream output, raw MCP traffic, source text, and user-project
command output out of the persisted report.

For repeatable native-agent outcome benchmarks, use the same script with a
suite file:

```bash
CTXHELM_RUN_REAL_CLIENT=1 bash scripts/e2e-agent-run.sh \
  --repo "$REPO" \
  --suite .ctxhelm/outcomes/tasks.json \
  --output .ctxhelm/e2e/agent-run-suite-claude.json
ctxhelm eval agent-run --report .ctxhelm/e2e/agent-run-suite-claude.json
```

Suite reports aggregate the native baseline, `prepare_task`, brief-pack,
standard-pack, and memory-guided standard-pack lanes across tasks. They are
still source-free: raw tasks are hashed, raw
prompts/transcripts/MCP traffic are not persisted, and the report stores only
target path labels, lane metrics, privacy flags, and sanitized request evidence.

Latest local real-client outcome refresh: 2026-06-05. Codex CLI `0.137.0`
passed the source-free five-lane agent-run matrix with four comparable
ctxhelm-assisted lanes, valid explicit-repo `prepare_task` and `get_pack`
calls, no forbidden commands, no client failures, no ctxhelm evidence misses,
and outcome claim `ctxhelm_improved`. Best lane `ctxhelm-memory` improved
target coverage by `+0.33`, reduced irrelevant reads by `2`, reduced command
executions by `14`, and reduced read-file count by `2`. See
`.planning/e2e/2026-06-05-phase237-codex-agent-run-outcome.md` and
`.ctxhelm/e2e/phase237-agent-run-codex.json`. Claude Code `2.1.163` remains
separately classified as unavailable because the preflight hit API status `429`
and reported `clientFailureKind = rate_limited`; see
`.planning/e2e/2026-06-05-phase236-agent-client-availability.md`.

Historical Claude workflow evidence is still useful but should not be mistaken
for current availability. The 2026-06-01 Claude Code `2.1.159` workflow passed
`scripts/e2e-claude-workflow.sh` with source-free explicit-repo `prepare_task`
and `get_pack` evidence; see
`.planning/e2e/2026-06-01-phase132-claude-workflow-eval.md`. The paired
agent-run refresh on the same date showed Claude Code `2.1.159` preserved
target coverage while the `ctxhelm-brief` lane reduced irrelevant reads from 5
to 2 and read-file count from 7 to 4; see
`.planning/e2e/2026-06-01-phase143-agent-run-outcome-harness.md`.

Cursor and OpenCode setup can be checked through generated artifacts plus deterministic protocol proof, including source-free context-area resource reads. Their optional real-client wrappers are separate from setup proof:

```bash
CTXHELM_RUN_CURSOR_REAL_CLIENT=1 bash scripts/smoke-cursor-real-client.sh
CTXHELM_RUN_OPENCODE_REAL_CLIENT=1 bash scripts/smoke-opencode-real-client.sh
```

Current local evidence: `scripts/smoke-opencode-real-client.sh` passes with
OpenCode `1.14.25` and records `prepare_task` plus `get_pack` calls with the
explicit repo. `scripts/smoke-cursor-real-client.sh` is available but currently
skips because `cursor agent status` reports not logged in. Required mode fails
instead of silently passing when those client preconditions are missing.

The setup-proof wrappers are:

```bash
bash scripts/smoke-cursor-mcp.sh
bash scripts/smoke-opencode-mcp.sh
```

They generate repo-local adapter files in a temporary repo, run `setup-check`,
run the deterministic MCP protocol smoke, and optionally write JSON evidence to
`CTXHELM_REAL_CLIENT_EVIDENCE_DIR`. The evidence explicitly marks
`realClientToolCalls: false` so setup proof is not confused with tool-call
transcript proof.

## Common Agent Flow

Use the same sequence across clients:

1. Call `prepare_task` with explicit `repo`, task text, mode, and active paths when known.
2. Let the agent use native file reads for the top targets and tests; discovering a path is not the same as consuming it.
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
ctxhelm init --repo "$REPO"
ctxhelm doctor --repo "$REPO"
ctxhelm setup-check --repo "$REPO"
```

Codex setup is copy/paste-oriented. ctxhelm does not edit `CODEX_HOME`, user config, or global MCP settings. Use an absolute `ctxhelm` binary path in Codex MCP config when GUI or shell `PATH` differs from your terminal.

## Claude Code Notes

Run:

```bash
ctxhelm init --repo "$REPO" --claude
ctxhelm doctor --repo "$REPO"
ctxhelm setup-check --repo "$REPO" --claude
```

The Claude path writes repo-local command guidance and a mergeable `.ctxhelm/adapters/claude-mcp.json` snippet. ctxhelm does not automatically write `.mcp.json`; the user or project maintainer chooses whether to merge the snippet.

## Cursor Notes

Run:

```bash
ctxhelm init --repo "$REPO" --cursor
ctxhelm doctor --repo "$REPO"
ctxhelm setup-check --repo "$REPO" --cursor
```

The Cursor path writes `.cursor/rules/ctxhelm.mdc`. That rule explains how to call ctxhelm through configured MCP surfaces, but setup validation only checks the generated file. It is not real-client request evidence.

## OpenCode Notes

Run:

```bash
ctxhelm init --repo "$REPO" --opencode
ctxhelm doctor --repo "$REPO"
ctxhelm setup-check --repo "$REPO" --opencode
```

The OpenCode path writes `.ctxhelm/adapters/opencode.jsonc.snippet`. It is a repo-local snippet for manual review or merge and is not applied to a global OpenCode configuration by ctxhelm.
