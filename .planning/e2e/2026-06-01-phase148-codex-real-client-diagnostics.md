# Phase 148: Codex Real-Client Diagnostic Evidence

## Goal

Make Codex real-client optional-skip evidence useful enough for production
release diagnosis without overclaiming machine-checkable Codex tool-call proof.

## Reproduction

The pre-fix Codex smoke reproduced the known gap:

- deterministic protocol gate passed
- Codex CLI `0.44.0` exited `1`
- stderr showed repeated `stream disconnected` retries
- server-side MCP log contained only initialization/listing requests
- evidence reported a generic optional skip without classifying the client
  failure

## Change

- `scripts/smoke-codex-mcp.sh` now records source-free client diagnostics on
  skipped Codex evidence:
  - `clientExitStatus`
  - `clientFailureKind`
  - `stderrLineCount`
  - `stderrSha256`
- Codex and Claude request summaries now include source-free MCP method
  accounting:
  - `methodCounts`
  - `initializeRequested`
  - `initializedNotification`
  - `toolsListRequested`
  - `resourcesListRequested`
  - `promptsListRequested`
- Docs and release-doc checks describe the diagnostic boundary and keep raw
  stderr, raw MCP traffic, prompts, task text, and source snippets out of
  persisted evidence.

## Public Archive Proof

Artifact: `.ctxhelm/e2e/phase148-public-real-client-diagnostics.json`

- Release binary: `ctxhelm 1.1.10`
- Release archive: `ctxhelm-v1.1.10-aarch64-apple-darwin.tar.gz`
- Archive SHA-256:
  `3eea9f0b85bf5973462c6cfff0dc6effe025059464640b954a540d9e739e3e8c`
- Claude Code `2.1.159`:
  - `status = passed`
  - `prepareTask = true`
  - `getPack = true`
  - `explicitRepoToolCallCount = 2`
  - `methodCounts.tools/call = 2`
- Codex CLI `0.44.0`:
  - `status = skipped`
  - `clientExitStatus = 1`
  - `clientFailureKind = stream_disconnected`
  - `requestLogLineCount = 3`
  - `methodCounts.initialize = 1`
  - `methodCounts.notifications/initialized = 1`
  - `methodCounts.tools/list = 1`
  - `explicitRepoToolCallCount = 0`

## Validation

- `bash -n scripts/smoke-codex-mcp.sh`
- `bash -n scripts/smoke-claude-mcp.sh`
- `bash scripts/check-release-docs.sh`
- `cargo fmt --check`
- `cargo test -p ctxhelm --test cli_compat real_client_smoke_scripts_have_contract_guards --locked`
- `cargo test -p ctxhelm --test release_packaging --locked`
- `bash scripts/smoke-public-real-clients.sh --repo thromel/ctxhelm --tag v1.1.10 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.10" --smoke-repo "$PWD" --output .ctxhelm/e2e/phase148-public-real-client-diagnostics.json`

## Notes

This does not make Codex real-client proof required. It makes the existing
optional Codex skip source-free and diagnosable, while preserving the rule that
real-client proof only passes when server-side instrumentation observes
explicit-repo `prepare_task` and `get_pack`.
