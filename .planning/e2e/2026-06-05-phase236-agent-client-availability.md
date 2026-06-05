# Phase 236: Agent Client Availability

## Goal

Separate current real-agent client availability from ctxhelm retrieval, pack,
and MCP protocol behavior before running another outcome-lift matrix.

## What Changed

- Added `scripts/e2e-agent-client-availability.sh`.
- The script checks Claude Code with a lightweight no-tool preflight.
- The script checks Codex CLI through the existing source-free MCP smoke and
  requires machine-checkable `prepare_task` and `get_pack` calls with an
  explicit repo.
- Added a release-packaging contract test for the availability script.
- Updated public docs and release assertions from the stale Codex CLI `0.44.0`
  skip boundary to the current Codex CLI `0.137.0` MCP evidence boundary.

## Evidence

Artifact:

- `.ctxhelm/e2e/phase236-agent-client-availability.json`

Summary:

- `status = passed`
- `clientCount = 2`
- `readyClientCount = 1`
- `unavailableClientCount = 1`
- `rateLimitedClientCount = 1`
- `streamDisconnectedClientCount = 0`
- `realAgentOutcomeCurrentlyRunnable = true`
- `recommendedResearchActions = ["run_real_agent_outcome_matrix"]`

Client results:

- Codex CLI `0.137.0`: available. The smoke recorded two explicit-repo ctxhelm
  tool calls: `prepare_task` and `get_pack` with `budget = "brief"`,
  `format = "json"`, and `recordTrace = false`.
- Claude Code `2.1.163`: unavailable because the preflight hit API status
  `429`. This is recorded as `rate_limited`, not as a ctxhelm protocol,
  retrieval, or pack failure.

Privacy:

- Raw prompts were not stored.
- Raw transcripts were not stored.
- Raw MCP traffic was not stored.
- Source text was not logged.
- Remote embeddings and remote reranking were not used.

## Validation

Commands:

```bash
bash -n scripts/e2e-agent-client-availability.sh
cargo test -p ctxhelm --test release_packaging agent_client_availability_script_contract --locked
git diff --check
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm bash scripts/e2e-agent-client-availability.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve paired agent-run lane matrix" \
  --output .ctxhelm/e2e/phase236-agent-client-availability.json
```

## Interpretation

The prior Codex blocker was client-version related. After replacing the old
Homebrew formula client with the current cask client, Codex produced
machine-checkable MCP evidence. The next R&D step should be a Codex-backed
real-agent outcome matrix, not another availability retry. Claude should be
retried only after the rate limit clears.
