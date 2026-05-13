# Feature Research: v1.1 Packaging & Adoption

**Domain:** Local-first agent-native adoption for ctxpack
**Researched:** 2026-05-13
**Confidence:** HIGH for current ctxpack state and Codex/Claude local CLI behavior; MEDIUM for Cursor/OpenCode config details because they should be rechecked during implementation against current docs and client versions.

## Scope Recommendation

v1.1 should make a completed v1 ctxpack easy to install, initialize, configure in real coding agents, and smoke-test. It should not change retrieval quality, add cloud services, or turn ctxpack into another coding agent.

The release should optimize for one user question: "Can I install this, wire it into my agent, prove MCP works, and get my first useful context pack in under ten minutes?"

## Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Copy-paste quickstart | Adoption fails if users must infer command order from the full README. | LOW | Put one happy path first: install, `ctxpack init`, `ctxpack smoke mcp`, agent setup, first `prepare-task`/`get-pack`. |
| Init setup report | `ctxpack init` already writes repo-local files, but users need a clear "what changed / what next" report. | LOW | Report generated files, skipped/unchanged files, exact MCP command, and next smoke command. Keep JSON output available for tests. |
| Agent profile generation | Users expect first-class setup for Codex, Claude Code, Cursor, and OpenCode. | MEDIUM | Keep one stable ctxpack core. Generate thin per-agent artifacts and snippets; do not fork planner behavior by agent. |
| Codex MCP guidance | Codex MCP config is user/global state, so setup must be explicit and inspectable. | LOW | Print `codex mcp add ctxpack -- ctxpack serve-mcp` and an equivalent config snippet. Do not mutate global Codex config by default. |
| Claude project setup | Claude Code supports project-scoped MCP config and slash-command workflows. | MEDIUM | Keep `.claude/commands/ctxpack-bugfix.md`; add a clearer project `.mcp.json` merge/write option or exact `claude mcp add-json --scope project` command. |
| Cursor rules setup | Cursor users expect repo-local rules under `.cursor/rules/`. | LOW | Keep `.cursor/rules/ctxpack.mdc` small and always focused on calling dynamic ctxpack/MCP context rather than injecting a repo map. |
| OpenCode setup snippet | OpenCode users need a mergeable local MCP config snippet. | MEDIUM | Keep `.ctxpack/adapters/opencode.jsonc.snippet`, but document where to merge it and provide a lint/check command. |
| Smoke command ladder | Users need to distinguish ctxpack brokenness from client auth/model issues. | MEDIUM | Make deterministic protocol smoke the hard gate, then optional Codex/Claude real-client smokes. Cursor/OpenCode can start with config/artifact validation unless a machine-checkable client smoke is available. |
| Doctor/troubleshooting output | MCP setup fails for PATH, cwd, repo arg, permissions, stale cache, or client config reasons. | MEDIUM | Add `ctxpack doctor` or `ctxpack smoke --explain` that checks binary path, repo root, inventory, MCP initialize/tools, and setup files. |
| First useful pack journey | The first success should show visible value, not just config success. | LOW | Quickstart should end with `ctxpack prepare-task "..." --repo . --path ...` and `ctxpack get-pack "..." --repo . --budget brief`. |
| Release artifact verification | Packaging is adoption work only if users can install without building from source. | MEDIUM | Gate release with binary install check, `ctxpack --help`, `ctxpack init`, protocol smoke, and generated config snapshot tests. |

## Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Agent-native first-run UX | ctxpack feels like it belongs inside existing agents instead of asking users to change workflows. | MEDIUM | Keep AGENTS.md, MCP, and thin adapter files as the product surface. CLI remains setup/debug automation. |
| Machine-checkable client proof | Claims about Codex/Claude integrations are credible only when tool calls are proven. | MEDIUM | Preserve server-side request logging in smoke scripts and require `prepare_task` plus `get_pack` with explicit `repo`. |
| Explicit non-mutation policy | Users trust setup more when global config writes are visible and optional. | LOW | Default to repo-local writes and printed commands/snippets. Add opt-in apply modes only after dry-run output is stable. |
| Configuration linting | Config generation is less valuable if users cannot validate it before opening an agent. | MEDIUM | Add checks for generated files, JSON syntax, command availability, and whether `ctxpack serve-mcp` starts from a wrong cwd. |
| One-page adoption docs | A concise quickstart lowers the cost of trying ctxpack on another repo. | LOW | Split "Quickstart" from "Reference"; keep troubleshooting close to setup commands. |

## Anti-Features

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Auto-editing global Codex/Claude/OpenCode config by default | Hidden MCP server registration is a trust problem and can affect unrelated projects. | Print exact commands/snippets; offer explicit `--apply` later only with dry-run and confirmation semantics. |
| Running user project tests in smoke commands | ctxpack is read-only and should not own project execution permissions. | Smoke ctxpack/MCP/client behavior; recommend validation commands in packs. |
| Retrieval-quality expansion in v1.1 | v1.1 is packaging/adoption; broad ranking work will blur acceptance criteria. | Defer to v1.2 Retrieval Quality Proof. |
| Standalone UI or chat app | Adds a new daily surface and conflicts with the agent-native product thesis. | Keep setup/docs/CLI/MCP focused on existing agents. |
| Cloud indexing or hosted setup | Violates local-first positioning and creates privacy review before adoption proof. | Ship local binaries, local config, local smokes. |
| Full Cursor/OpenCode real-client automation without reliable evidence | A passing transcript without tool-call proof can become a false compatibility claim. | Start with generated-artifact validation; add real-client smoke only when machine-checkable. |
| Large static instruction files | Token-heavy rules become stale and compete with agent instructions. | Keep static guidance short; point to MCP `prepare_task` and `get_pack`. |

## Quickstart Journey

Recommended docs flow:

```bash
# 1. Install or build ctxpack
ctxpack --help

# 2. Initialize the repository
ctxpack init --repo . --cursor --claude --opencode

# 3. Prove the local MCP server works without any agent auth
ctxpack smoke mcp --repo .

# 4. Add one agent integration explicitly
codex mcp add ctxpack -- ctxpack serve-mcp
# or: claude mcp add-json --scope project ctxpack '{"command":"ctxpack","args":["serve-mcp"]}'

# 5. Optional real-client proof where supported
ctxpack smoke codex --repo .
ctxpack smoke claude --repo .

# 6. Get first useful context
ctxpack prepare-task "fix the failing auth session test" --repo .
ctxpack get-pack "fix the failing auth session test" --repo . --budget brief
```

Implementation note: existing scripts already cover protocol, Codex, and Claude smoke paths. v1.1 should wrap or document them as user-facing commands so users do not need to discover `scripts/`.

## Agent Setup Matrix

| Agent | v1.1 Output | Default Write Scope | Smoke Gate | Recommendation |
|-------|-------------|---------------------|------------|----------------|
| Codex CLI | Printed `codex mcp add ctxpack -- ctxpack serve-mcp` plus config snippet. | No global mutation by default. | Protocol smoke, optional real-client smoke with isolated `CODEX_HOME`. | Treat Codex setup as explicit user action. Current local CLI supports `codex mcp add`. |
| Claude Code | `.claude/commands/ctxpack-bugfix.md` plus `.mcp.json` snippet or explicit `claude mcp add-json --scope project`. | Repo/project-local. | Protocol smoke, optional real-client smoke with strict MCP config. | Make project setup the smoothest path because Claude supports project-scoped MCP. |
| Cursor | `.cursor/rules/ctxpack.mdc`. | Repo-local. | Generated-file lint plus protocol smoke. | Keep the rule tiny; instruct Cursor to use MCP/dynamic context when configured. |
| OpenCode | `.ctxpack/adapters/opencode.jsonc.snippet` and docs for merge location. | Repo-local snippet by default. | Generated-file lint plus protocol smoke. | Avoid claiming full client proof until an automated OpenCode MCP smoke can verify tool calls. |

## Feature Dependencies

```text
Release install artifact
  -> ctxpack --help
  -> ctxpack init report
  -> generated adapter files/snippets
  -> protocol smoke
  -> optional real-client smokes
  -> first prepare-task/get-pack quickstart
```

```text
Agent config generation
  -> exact command/snippet output
  -> config lint
  -> no hidden global mutation
  -> troubleshooting docs
```

## v1.1 MVP Recommendation

Prioritize:

1. **Quickstart and init report** - the shortest path from install to first pack.
2. **Agent setup matrix** - Codex, Claude, Cursor, OpenCode generation with explicit write scope.
3. **Smoke command ladder** - protocol hard gate, optional Codex/Claude real-client proof, config lint for Cursor/OpenCode.
4. **Troubleshooting/doctor** - PATH, repo arg, wrong cwd, MCP startup, generated config, and stale inventory diagnostics.

Defer:

- Cursor/OpenCode real-client smoke unless tool-call evidence can be captured reliably.
- Any retrieval-ranking changes; v1.1 should not move eval baselines.
- Hosted installers, UI, team policy, and cloud indexing.

## Acceptance Gates

| Gate | Required Proof |
|------|----------------|
| Install smoke | Fresh shell can run `ctxpack --help`. |
| Init smoke | Temp repo init creates/updates only expected repo-local files and prints next steps. |
| Config snapshots | Generated Codex/Claude/Cursor/OpenCode artifacts are snapshot-tested. |
| Protocol smoke | MCP initialize/tools plus `prepare_task`/`get_pack` pass from wrong cwd with explicit `repo`. |
| Codex smoke | Optional/required mode records `prepare_task` and `get_pack` tool calls with explicit `repo`. |
| Claude smoke | Optional/required mode records `prepare_task` and `get_pack` tool calls with explicit `repo`. |
| Docs smoke | Quickstart commands are copy-pasteable and match current CLI flags. |

## Sources

- Local project state: `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `README.md`, `crates/ctxpack-core/src/init.rs`, `scripts/smoke-mcp-protocol.sh`, `scripts/smoke-codex-mcp.sh`, `scripts/smoke-claude-mcp.sh` (HIGH confidence).
- Local CLI checks on 2026-05-13: `codex-cli 0.130.0` supports `codex mcp add <name> -- <command>`; Claude Code `2.1.140` supports `claude mcp add-json --scope <local|user|project>`; OpenCode `1.14.25` exposes `opencode mcp add/list/auth/logout/debug` (HIGH confidence for this machine).
- Claude Code MCP docs: https://code.claude.com/docs/en/mcp (HIGH confidence for Claude MCP setup concepts).
- Cursor rules docs: https://docs.cursor.com/context/rules (MEDIUM confidence; verify exact MDC fields during implementation).
- OpenCode config/MCP docs: https://opencode.ai/docs/config/ and https://opencode.ai/docs/mcp-servers/ (MEDIUM confidence; verify schema during implementation).
- Model Context Protocol server concepts: https://modelcontextprotocol.io/docs/learn/server-concepts (HIGH confidence for tools/resources/prompts framing).
