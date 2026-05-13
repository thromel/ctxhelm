# Troubleshooting

This reference covers local install, setup, and MCP startup failures. ctxpack remains local-first and read-only: it does not edit source files, run project tests, install dependencies, or mutate global agent configuration.

## `ctxpack: command not found`

**Likely cause:** the binary is not on the `PATH` used by your shell or agent client.

**Fix:**

```bash
ctxpack --version
ctxpack --help
```

If those fail, install the release binary into a directory on `PATH`, for example `~/.local/bin`, or call the binary by absolute path.

GUI-launched agents can have a different PATH than your terminal. When configuring MCP clients, prefer an absolute `ctxpack` binary path if startup fails from the GUI.

## Absolute MCP Binary Paths

**Symptom:** the MCP client says it cannot start `ctxpack`, exits immediately, or reports no tools.

**Likely cause:** the client process cannot resolve `ctxpack` from its PATH.

**Fix:** configure the MCP server command with an absolute path:

```json
{
  "mcpServers": {
    "ctxpack": {
      "command": "/Users/you/.local/bin/ctxpack",
      "args": ["serve-mcp"]
    }
  }
}
```

Run the same path directly first:

```bash
/Users/you/.local/bin/ctxpack --version
/Users/you/.local/bin/ctxpack --help
```

## `CTXPACK_HOME` And Local State

By default, ctxpack stores private local state under `~/.ctxpack`. This can include safe inventory cache files and source-free trace records. Set `CTXPACK_HOME` to isolate state for tests, demos, or temporary work:

```bash
export CTXPACK_HOME="$(mktemp -d)"
ctxpack prepare-task "fix auth bug" --repo /path/to/repo --mode bug-fix
```

If a smoke or test uses a custom home, clean that directory instead of `~/.ctxpack`.

## Uninstall And State Cleanup

To uninstall the release binary, remove the installed executable:

```bash
rm -f ~/.local/bin/ctxpack
```

To remove default local state:

```bash
rm -rf ~/.ctxpack
```

For custom state cleanup, remove the directory named by `CTXPACK_HOME`. Do not remove project source directories; ctxpack state is separate from your repository.

## wrong cwd Or Wrong Repository

**Symptom:** ctxpack finds no repository, reports unexpected files, or returns context for the wrong project.

**Likely cause:** the command or MCP server started from a different working directory than the repository you intended.

**Fix:** pass an explicit `--repo` on CLI commands:

```bash
ctxpack prepare-task "fix requireSession bug" --repo /path/to/repo --mode bug-fix
ctxpack get-pack "fix requireSession bug" --repo /path/to/repo --mode bug-fix --budget brief
```

For MCP tools, pass the `repo` argument explicitly:

```json
{
  "task": "fix requireSession bug",
  "repo": "/path/to/repo",
  "mode": "bug_fix"
}
```

## MCP Startup Failures

**Symptom:** an MCP client starts but shows no ctxpack tools, hangs on startup, or exits before initialization.

Check these first:

- The configured command points to the installed `ctxpack` binary.
- The configured args include `serve-mcp`.
- The configured command works with `--version` and `--help`.
- The process has permission to read the repository and write its ctxpack home.
- The client uses a clean stdio channel. MCP JSON-RPC responses are written to stdout; avoid wrapper scripts that print banners or debug text to stdout.

If you need wrapper diagnostics, write them to stderr. stdout cleanliness matters because MCP clients parse stdout as protocol messages.

## Permissions And Read-Only Homes

ctxpack needs read access to the target repository and write access to `CTXPACK_HOME` or `~/.ctxpack` for local inventory and traces. If the home directory is read-only, set a writable temporary home:

```bash
export CTXPACK_HOME=/tmp/ctxpack-home
mkdir -p "$CTXPACK_HOME"
```

ctxpack does not require permission to modify your source files.

## `setup-check` Scope

`setup-check` validates repo-local generated artifacts such as `AGENTS.md`, `.ctxpack/ctxpack.toml`, `.cursor/rules/ctxpack.mdc`, `.claude/commands/ctxpack-bugfix.md`, and adapter snippets when requested.

It does not run real agent clients, does not prove a client called MCP tools, and does not mutate global Codex, Claude, Cursor, or OpenCode configuration.

## Session-Scoped Pack Resources

`prepare_task` returns pack resource URIs for MCP clients. Those URIs are session-scoped and are available during the same MCP server process that returned them.

**Symptom:** reading an old `ctxpack://pack/...` URI fails after reconnecting or restarting the MCP server.

**Fix:** use `get_pack` as the durable reconnect path:

```json
{
  "task": "fix requireSession bug",
  "repo": "/path/to/repo",
  "mode": "bug_fix",
  "budget": "brief",
  "format": "json"
}
```

## Deterministic Proof Versus Client Proof

The deterministic protocol proof talks directly to `ctxpack serve-mcp` and checks machine-readable `prepare_task` and `get_pack` responses. Optional real-client proof is separate and version-specific. v1.1 does not claim machine-checkable Cursor or OpenCode tool-call proof.
