# Phase 106 E2E: Real-Client Request Evidence Hardening

## Goal

Harden Codex CLI and Claude Code real-client MCP smoke evidence so successful
client runs are auditable without storing raw MCP traffic, prompts, task text, or
source snippets.

## Changes

- `scripts/smoke-codex-mcp.sh` and `scripts/smoke-claude-mcp.sh` now preserve
  the existing compatibility fields: `client`, `clientVersion`,
  `ctxpackVersion`, `repo`, `deterministicProtocol`,
  `deterministicContextAreaResourceRead`, `prepareTask`, `getPack`, and
  `required`.
- Evidence now adds `requestEvidenceSchemaVersion`,
  `serverSideRequestLog`, `requestLogSha256`, `requestLogLineCount`,
  `explicitRepoToolCallCount`, and sanitized `observedToolCalls`.
- Evidence-directory runs write a sanitized `*-mcp-request-summary.json`
  sidecar and reference it by basename in `requestSummaryFile`.
- The observed tool-call summary stores only tool names and source-free argument
  facts such as repo match, task presence, `budget`, `format`, and
  `recordTraceFalse`.
- Claude semantic smoke mode also records per-call `semanticMatched` when
  semantic provider/model/dimension requirements are enabled.
- The Codex wrapper now adapts to older `codex exec` builds by using
  `--ephemeral` and `--ignore-user-config` only when the installed client
  supports them; older clients receive an isolated `CODEX_HOME` to avoid parsing
  incompatible user config.

## Source-Free Evidence Shape

Claude Code real-client smoke produced this sanitized request summary:

```json
{
  "requestEvidenceSchemaVersion": "ctxpack-real-client-evidence-v2",
  "serverSideRequestLog": true,
  "requestLogLineCount": 7,
  "explicitRepoToolCallCount": 2,
  "observedToolCalls": [
    {
      "name": "prepare_task",
      "repoMatched": true,
      "hasTask": true
    },
    {
      "name": "get_pack",
      "repoMatched": true,
      "hasTask": true,
      "budget": "brief",
      "format": "json",
      "recordTraceFalse": true
    }
  ],
  "requestSummaryFile": "claude-mcp-request-summary.json"
}
```

The committed proof intentionally omits raw request logs, prompt text, task
text, source snippets, and local request-log paths.

## Validation

Passed:

```bash
cargo fmt --check
git diff --check
bash scripts/check-release-docs.sh
CARGO_TARGET_DIR=/tmp/ctxpack-phase106-target cargo test -p ctxpack --test cli_compat real_client_smoke_scripts_have_contract_guards -- --nocapture
CTXPACK_BIN=/tmp/ctxpack-phase106-target/debug/ctxpack CTXPACK_SKIP_REAL_CLIENT=1 bash scripts/smoke-codex-mcp.sh
CTXPACK_BIN=/tmp/ctxpack-phase106-target/debug/ctxpack CTXPACK_SKIP_REAL_CLIENT=1 bash scripts/smoke-claude-mcp.sh
CTXPACK_BIN=/tmp/ctxpack-phase106-target/debug/ctxpack CTXPACK_RUN_REAL_CLIENT=1 CTXPACK_REAL_CLIENT_EVIDENCE_DIR=/tmp/ctxpack-phase106-real-client-evidence CTXPACK_SMOKE_REPO="$PWD" bash scripts/smoke-claude-mcp.sh
CARGO_TARGET_DIR=/tmp/ctxpack-phase106-target cargo test --workspace --no-fail-fast
CARGO_TARGET_DIR=/tmp/ctxpack-phase106-target cargo run -p ctxpack -- --help
```

Observed real clients:

- Claude Code `2.1.158` passed the real-client MCP smoke and wrote sanitized
  evidence with `explicitRepoToolCallCount = 2`.
- Codex CLI `0.44.0` did not produce real-client tool-call evidence in this
  environment because the stream disconnected after retries. The wrapper no
  longer fails on unsupported `--ephemeral` or `--ignore-user-config` options;
  optional Codex evidence remains skipped unless `CTXPACK_REQUIRE_REAL_CLIENT=1`
  is set.

Deferred full gate:

- None.
