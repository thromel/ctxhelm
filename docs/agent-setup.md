# Agent Setup Matrix

ctxhelm is an agent-native, read-only context broker. `ctxhelm setup repo` generates repo-local guidance and snippets that help existing agents call `prepare_task`, read returned target files natively, and request `get_pack` only when they need more context. Treat path discovery as a weaker signal than file consumption: agents should read the files they rely on before editing or answering.

## Support Matrix

| Agent | Generated artifact/snippet | Default write scope | Mutates global config by default | setup-check coverage | deterministic protocol proof | Optional real-client proof status | Verified-version/evidence notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Codex CLI | Managed `AGENTS.md` section through `ctxhelm setup repo`, plus printed MCP setup guidance | Repo-local guidance only; user copies any MCP config manually | No global config mutation | Verifies managed `AGENTS.md` and `.ctxhelm/ctxhelm.toml` | Supported through direct JSON-RPC/MCP smoke against `ctxhelm serve-mcp` | Optional Codex smoke can require server-side `prepare_task` and `get_pack` request-log evidence; current local outcome proof passes with explicit-repo tool-call evidence | Local command available: Codex CLI `0.137.0`; current local matrix records five source-free lanes, four comparable ctxhelm lanes, no forbidden commands, and outcome claim `ctxhelm_improved` |
| Claude Code | `.claude/commands/ctxhelm-bugfix.md`, `.ctxhelm/adapters/claude-mcp.json`, and project-local `.mcp.json` through `ctxhelm setup repo` or `ctxhelm setup claude` | Repo/project-local command and MCP config only | No global config mutation; project MCP write is explicit and repo-local | Verifies Claude command, adapter snippet, and project `.mcp.json` readiness when `--claude` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxhelm serve-mcp` | Optional Claude smoke and workflow eval can require server-side `prepare_task` and `get_pack` request-log evidence; current availability preflight is rate-limited | Local command available: Claude Code `2.1.163`; current availability proof classifies API status `429` separately from ctxhelm protocol and retrieval behavior |
| Cursor | `.cursor/rules/ctxhelm.mdc` through `ctxhelm setup repo` or `ctxhelm init --cursor` | Repo-local rule file | No global config mutation | Verifies Cursor rule when `--cursor` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxhelm serve-mcp` | Optional Cursor Agent CLI smoke can require server-side `prepare_task` and `get_pack` request-log evidence; current local proof is skipped because Cursor Agent is not logged in | Local command available: Cursor `3.6.21`; version presence alone is not tool-call proof |
| OpenCode | `.ctxhelm/adapters/opencode.jsonc.snippet` through `ctxhelm setup repo` or `ctxhelm init --opencode` | Repo-local snippet for manual merge | No global config mutation | Verifies OpenCode snippet when `--opencode` is requested | Supported through direct JSON-RPC/MCP smoke against `ctxhelm serve-mcp` | Optional OpenCode smoke passes locally with server-side `prepare_task` and `get_pack` request-log evidence | Local command available: OpenCode `1.14.25`; current proof records two explicit-repo MCP tool calls |

## Proof Taxonomy

**Generated artifact checks** are read-only setup validation. `ctxhelm setup-check --repo "$REPO" --cursor --claude --opencode` confirms expected repo-local files exist and contain recognizable ctxhelm guidance. With `--claude`, it also checks whether project `.mcp.json` registers `mcpServers.ctxhelm` with an absolute binary path and `["serve-mcp"]` args. Missing `.mcp.json` is a warning so the manual `init --claude` path remains inspectable; malformed or unsafe existing project MCP config is a failure. It does not run real agent clients.

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

Latest local real-client outcome refresh: 2026-06-06. Codex CLI `0.137.0`
passes both the source-free multi-task suite and the governor/release R&D
regression task. The Phase 245 suite reports two comparison-eligible tasks,
eight comparable ctxhelm lanes, no forbidden commands, no client failures, no
ctxhelm evidence misses, no evidence-only targets, no under-read targets, and
outcome claim `ctxhelm_improved`: baseline average target-read coverage is
`0.75`, while every ctxhelm lane reaches `1.00`. The Phase 250 governor run
then fixes a real under-read regression and reports best lane
`ctxhelm-standard`, no ctxhelm evidence misses, no under-read targets, and
`1.00` target-read coverage in all ctxhelm lanes. See
`.planning/e2e/2026-06-05-phase245-codex-agent-run-suite.md`,
`.planning/e2e/2026-06-06-phase250-governor-artifact-retrieval.md`,
`.ctxhelm/e2e/phase245-agent-run-codex-suite-real-bounded-final.json`, and
`.ctxhelm/e2e/phase250-agent-run-codex-governor-rd-after.json`.

Latest Claude Code workflow refresh: 2026-06-06. Claude Code workflow proof
passes with source-free explicit-repo MCP evidence and local-only privacy in
`.ctxhelm/e2e/phase250-claude-workflow-refresh.json`. That current workflow
proof should be separated from the older paired Claude outcome refresh: the
2026-06-05 availability preflight for Claude Code `2.1.163` hit API status
`429` and reported `clientFailureKind = rate_limited`, which is client
availability evidence rather than ctxhelm retrieval failure.

Historical Codex outcome evidence remains part of the release-doc contract. The
first current Codex CLI `0.137.0` outcome refresh is recorded in
`.planning/e2e/2026-06-05-phase237-codex-agent-run-outcome.md` and
`.ctxhelm/e2e/phase237-agent-run-codex.json`.

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

## One-Command Repo Setup

Run:

```bash
ctxhelm setup repo --repo "$REPO"
```

This is the default secure onboarding path. It writes repo-local `AGENTS.md`,
`.ctxhelm/ctxhelm.toml`, Cursor guidance, Claude guidance, a Claude MCP
snippet, an OpenCode snippet, and project-local `.mcp.json` for local MCP. It
uses an absolute ctxhelm binary path and does not mutate global Codex, Claude,
Cursor, or OpenCode config.

Preview without writing:

```bash
ctxhelm setup repo --repo "$REPO" --dry-run
```

Dry-run setup is a read-only preflight: it lists the files that would be
written, runs `setup-check` against the current repo-local artifacts, and keeps
the exit status successful even when the current repo is not configured yet.
Use this when you want a safe setup plan before letting ctxhelm write files.

Automation can request a source-free setup report:

```bash
ctxhelm setup repo --repo "$REPO" --dry-run --format json
```

The JSON dry-run report includes `setupCheck`, `privacyStatus`, and
`unsupportedActions`, so automation can distinguish missing repo-local setup
from privacy or global-config mutation concerns without reading source text.

Use agent-specific setup only when you intentionally want fewer repo-local
artifacts.

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
ctxhelm setup claude --repo "$REPO"
```

This is the easiest secure Claude path. It initializes the repo-local Claude
command and adapter snippet, writes or merges project `.mcp.json` with only
`mcpServers.ctxhelm`, uses an absolute ctxhelm binary path, runs setup
validation, and does not mutate global Claude Code config.

Preview without writing:

```bash
ctxhelm setup claude --repo "$REPO" --dry-run
```

The Claude dry-run path also embeds the read-only setup-check result, so a user
can see whether `.claude/commands/ctxhelm-bugfix.md`,
`.ctxhelm/adapters/claude-mcp.json`, and `.mcp.json` are already ready before
writing repo-local files.

Manual setup is still available:

```bash
ctxhelm init --repo "$REPO" --claude
ctxhelm doctor --repo "$REPO"
ctxhelm setup-check --repo "$REPO" --claude
```

The manual Claude path writes repo-local command guidance and a mergeable
`.ctxhelm/adapters/claude-mcp.json` snippet. Use it when you want to inspect or
merge MCP config yourself.

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
