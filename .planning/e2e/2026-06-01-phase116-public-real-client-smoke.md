# Phase 116 E2E: Public Archive Real-Client Smoke

Date: 2026-06-01

## Objective

Refresh optional Codex CLI and Claude Code real-client evidence against the
published `v1.1.0` GitHub archive binary, not a build-tree binary.

## Command

```bash
bash scripts/smoke-public-real-clients.sh \
  --repo thromel/ctxhelm \
  --tag v1.1.0 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxhelm 1.1.0" \
  --smoke-repo "$(pwd)" \
  --output .ctxhelm/e2e/phase116-public-real-client-smoke.json
```

## Result

The public release archive was downloaded from GitHub, checksum-verified,
archive-verified, extracted, and used as the selected `CTXHELM_BIN`.

Summary artifact:

- `.ctxhelm/e2e/phase116-public-real-client-smoke.json`

Evidence sidecars:

- `.ctxhelm/e2e/phase116-public-real-client-smoke-evidence/codex-mcp-evidence.json`
- `.ctxhelm/e2e/phase116-public-real-client-smoke-evidence/codex-mcp-request-summary.json`
- `.ctxhelm/e2e/phase116-public-real-client-smoke-evidence/claude-mcp-evidence.json`
- `.ctxhelm/e2e/phase116-public-real-client-smoke-evidence/claude-mcp-request-summary.json`

## Client Verdicts

| Client | Version | Status | Tool-call evidence |
| --- | --- | --- | --- |
| Codex CLI | `codex-cli 0.44.0` | `skipped` | `prepare_task=false`, `get_pack=false`, `explicitRepoToolCallCount=0` |
| Claude Code | `2.1.158 (Claude Code)` | `passed` | `prepare_task=true`, `get_pack=true`, `explicitRepoToolCallCount=2` |

Codex was installed but exited before producing machine-checkable tool-call
evidence. Because real-client proof is optional unless
`CTXHELM_REQUIRE_REAL_CLIENT=1`, this is recorded as source-free skip evidence,
not as a product failure or as passed Codex proof.

Claude Code produced server-side request-log evidence for explicit-repo
`prepare_task` and `get_pack` calls through the released `ctxhelm 1.1.0` binary.

## Privacy And Boundary

- Raw MCP traffic is not persisted.
- Evidence records request-log SHA-256, line count, explicit repo tool-call
  count, and sanitized observed tool-call metadata.
- The script does not install globally, mutate agent configuration, publish,
  upload, create tags, or run user project tests.
