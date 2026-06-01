# Agent Preview

`ctxhelm agent preview` shows how ctxhelm expects an existing coding agent to
consume a task-specific plan or pack.

```bash
ctxhelm agent preview "fix login redirect" --repo /path/to/repo
ctxhelm agent preview "fix login redirect" --target-agent codex --format json
ctxhelm agent preview "fix login redirect" --target-agent claude-code
ctxhelm agent preview "fix login redirect" --target-agent cursor
ctxhelm agent preview "fix login redirect" --target-agent opencode
ctxhelm agent preview "fix login redirect" --target-agent generic
```

The default target is `all`, which emits previews for Codex CLI, Claude Code,
Cursor, OpenCode, and a generic MCP client.

## What It Shows

Each preview includes:

- target agent name
- same-session pack resource URI
- MCP tools such as `prepare_task`, `get_pack`, `related`, and `related_tests`
- MCP resources such as `ctxhelm://repo/summary` and `ctxhelm://pack/...`
- `AGENTS.md` guidance
- native rule, command, or adapter snippet paths where applicable
- recommended next steps
- ownership boundary between ctxhelm and the target agent

## Boundary

Agent previews are source-free. They do not include raw source text, safe pack
snippets, prompts, terminal logs, model transcripts, global agent configs, or
cloud payloads.

The preview makes this division explicit:

- ctxhelm suggests files, tests, context packs, and validation commands.
- the coding agent reads files with native tools.
- the coding agent edits files and runs shell commands through its permission
  model.
- source-bearing content appears only when the user explicitly exports or
  requests a context pack.
- cloud embeddings and cloud reranking stay disabled by default.

## Release Smoke

Maintainers can run:

```bash
bash scripts/smoke-agent-preview.sh
```

The smoke verifies all target agents are present, MCP tools/resources are
visible, native guidance paths are shown, source-free flags remain false, and a
source sentinel does not leak into preview artifacts.
