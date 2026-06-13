# Phase 340: Claude Setup UX

## Summary

Claude Code setup now has a one-command secure path:

```bash
ctxhelm setup claude --repo <repo>
```

## Change

The command:

- initializes repo-local Claude guidance
- writes `.claude/commands/ctxhelm-bugfix.md`
- writes `.ctxhelm/adapters/claude-mcp.json`
- writes or merges project-local `.mcp.json`
- only owns `mcpServers.ctxhelm`
- uses an absolute ctxhelm binary path
- runs setup validation
- leaves global Claude Code config untouched

`--dry-run` previews the same actions without writing files.

## Security Boundary

This phase keeps setup project-local. It does not write user-global Claude
configuration, shell startup files, credentials, or remote state. Existing MCP
servers in `.mcp.json` are preserved, and symlinked `.mcp.json` paths are
rejected before write.

## Acceptance

- `ctxhelm setup claude --repo <repo>` succeeds on a fixture repo.
- Existing `.mcp.json` MCP entries are preserved.
- `mcpServers.ctxhelm.command` is absolute.
- `mcpServers.ctxhelm.args` is `["serve-mcp"]`.
- `ctxhelm setup claude --repo <repo> --dry-run` writes no files.
- Docs present the one-command Claude path first.
