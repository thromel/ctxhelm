# Troubleshooting

This reference covers local install, setup, and MCP startup failures. ctxhelm remains local-first and read-only: it does not edit source files, run project tests, install dependencies, or mutate global agent configuration.

## `ctxhelm: command not found`

**Likely cause:** the binary is not on the `PATH` used by your shell or agent client.

**Fix:**

```bash
ctxhelm --version
ctxhelm --help
```

If those fail, install the release binary into a directory on `PATH`, for example `~/.local/bin`, or call the binary by absolute path.

GUI-launched agents can have a different PATH than your terminal. When configuring MCP clients, prefer an absolute `ctxhelm` binary path if startup fails from the GUI.

Run the install doctor to capture all binary-path checks in one place:

```bash
ctxhelm doctor --repo /path/to/repo --binary "$(command -v ctxhelm)"
```

## Absolute MCP Binary Paths

**Symptom:** the MCP client says it cannot start `ctxhelm`, exits immediately, or reports no tools.

**Likely cause:** the client process cannot resolve `ctxhelm` from its PATH.

**Fix:** configure the MCP server command with an absolute path:

```json
{
  "mcpServers": {
    "ctxhelm": {
      "command": "/Users/you/.local/bin/ctxhelm",
      "args": ["serve-mcp"]
    }
  }
}
```

Run the same path directly first:

```bash
/Users/you/.local/bin/ctxhelm --version
/Users/you/.local/bin/ctxhelm --help
```

## `CTXHELM_HOME` And Local State

By default, ctxhelm stores private local state under `~/.ctxhelm`. This can include safe inventory cache files and source-free trace records. Set `CTXHELM_HOME` to isolate state for tests, demos, or temporary work:

```bash
export CTXHELM_HOME="$(mktemp -d)"
ctxhelm prepare-task "fix auth bug" --repo /path/to/repo --mode bug-fix
```

If a smoke or test uses a custom home, clean that directory instead of `~/.ctxhelm`.

## Uninstall And State Cleanup

To uninstall the release binary, remove the installed executable:

```bash
rm -f ~/.local/bin/ctxhelm
```

To remove default local state:

```bash
rm -rf ~/.ctxhelm
```

For custom state cleanup, remove the directory named by `CTXHELM_HOME`. Do not remove project source directories; ctxhelm state is separate from your repository.

## Stale Binary Or Upgrade Mismatch

**Symptom:** `ctxhelm --version` is older than the release archive you unpacked,
or the agent still starts an older binary.

**Fix:** locate the active binary and verify it against the release manifest:

```bash
command -v ctxhelm
ctxhelm --version
ctxhelm doctor \
  --repo /path/to/repo \
  --binary "$(command -v ctxhelm)" \
  --release-manifest /path/to/ctxhelm-v1.1.12-aarch64-apple-darwin.manifest.json
```

If `doctor` reports a manifest version mismatch, update the binary path used by
your shell or MCP client. GUI clients often keep a stale absolute command path.

## wrong cwd Or Wrong Repository

**Symptom:** ctxhelm finds no repository, reports unexpected files, or returns context for the wrong project.

**Likely cause:** the command or MCP server started from a different working directory than the repository you intended.

**Fix:** pass an explicit `--repo` on CLI commands:

```bash
ctxhelm prepare-task "fix requireSession bug" --repo /path/to/repo --mode bug-fix
ctxhelm get-pack "fix requireSession bug" --repo /path/to/repo --mode bug-fix --budget brief
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

**Symptom:** an MCP client starts but shows no ctxhelm tools, hangs on startup, or exits before initialization.

Check these first:

- The configured command points to the installed `ctxhelm` binary.
- The configured args include `serve-mcp`.
- The configured command works with `--version` and `--help`.
- The process has permission to read the repository and write its ctxhelm home.
- The client uses a clean stdio channel. MCP JSON-RPC responses are written to stdout; avoid wrapper scripts that print banners or debug text to stdout.

If you need wrapper diagnostics, write them to stderr. stdout cleanliness matters because MCP clients parse stdout as protocol messages.

## Permissions And Read-Only Homes

ctxhelm needs read access to the target repository and write access to `CTXHELM_HOME` or `~/.ctxhelm` for local inventory and traces. If the home directory is read-only, set a writable temporary home:

```bash
export CTXHELM_HOME=/tmp/ctxhelm-home
mkdir -p "$CTXHELM_HOME"
```

ctxhelm does not require permission to modify your source files.

## `setup-check` Scope

`setup-check` validates repo-local generated artifacts such as `AGENTS.md`, `.ctxhelm/ctxhelm.toml`, `.cursor/rules/ctxhelm.mdc`, `.claude/commands/ctxhelm-bugfix.md`, and adapter snippets when requested.

It does not run real agent clients, does not prove a client called MCP tools, and does not mutate global Codex, Claude, Cursor, or OpenCode configuration.

Use `ctxhelm doctor` before `setup-check` when the failure may be install,
upgrade, binary path, release manifest, or local state compatibility rather
than generated guidance files.

## Session-Scoped Pack Resources

`prepare_task` returns pack resource URIs for MCP clients. Those URIs are session-scoped and are available during the same MCP server process that returned them.

**Symptom:** reading an old `ctxhelm://pack/...` URI fails after reconnecting or restarting the MCP server.

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

The deterministic protocol proof talks directly to `ctxhelm serve-mcp` and checks machine-readable `prepare_task` and `get_pack` responses. Optional real-client proof is separate and version-specific. Codex, Claude Code, Cursor Agent CLI, and OpenCode wrappers pass only when server-side instrumentation records explicit-repo `prepare_task` and `get_pack` calls; unavailable auth/provider state is recorded as a source-free skip.

When Codex or Claude real-client evidence is enabled with
`CTXHELM_REAL_CLIENT_EVIDENCE_DIR`, inspect `*-mcp-evidence.json` first. The
stable evidence includes `requestLogSha256`, `requestLogLineCount`,
MCP request `methodCounts`, `explicitRepoToolCallCount`, `observedToolCalls`,
and `requestSummaryFile`. Codex skip evidence also includes
`clientFailureKind`, `clientExitStatus`, `stderrSha256`, and
`stderrLineCount`; for example, `stream_disconnected` means the Codex client
failed before producing machine-checkable tool calls, while the deterministic
protocol check may still pass. Cursor and OpenCode real-client wrappers write
`cursor-real-client-evidence.json` and `opencode-real-client-evidence.json`
when `CTXHELM_REAL_CLIENT_EVIDENCE_DIR` is set.
The request summary sidecar is sanitized and source-free; it is intended to prove which ctxhelm tools were
observed without preserving raw MCP traffic, raw stderr, task text, or source
snippets.
