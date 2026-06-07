# Phase 304 - Claude Outcome Availability Refresh

## Scope

Phase 303 rejected further retention-separator relaxation and moved the next
high-value R&D branch toward fresh agent outcome proof. Phase 304 refreshes the
Claude Code outcome state before changing retrieval again.

No harness code changed in this phase. The existing source-free availability
and paired agent-run scripts already separate client availability from
retrieval-quality evidence.

## Commands

```bash
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
CTXHELM_AGENT_AVAILABILITY_TIMEOUT_SECONDS=60 \
bash scripts/e2e-agent-client-availability.sh \
  --repo "$(pwd)" \
  --task "Refresh Claude R&D breadth-suite outcome proof" \
  --output .ctxhelm/e2e/phase304-agent-client-availability.json
```

```bash
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=120 \
CTXHELM_AGENT_RUN_PREFLIGHT_TIMEOUT_SECONDS=60 \
bash scripts/e2e-agent-run.sh \
  --repo "$(pwd)" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --output .ctxhelm/e2e/phase304-agent-run-claude-rd-breadth-suite.json
```

## Result

The client availability report is source-free and passes as an availability
snapshot:

| Client | Version | Status | Ready | Failure |
| --- | --- | --- | --- | --- |
| Claude Code | `2.1.163 (Claude Code)` | `unavailable` | `false` | `rate_limited` |
| Codex CLI | `codex-cli 0.137.0` | `available` | `true` | `none` |

The same four-task R&D breadth suite used by Phases 251 and 254 was rerun
against Claude Code. It remains degraded:

| Metric | Value |
| --- | ---: |
| status | `degraded` |
| suite hash | `d04f86b3b8fb792a6d8dad7b493f728b1b78901a63a473dd004f2247b2b54afe` |
| task count | `4` |
| comparison-eligible tasks | `0` |
| comparable ctxhelm lanes | `0` |
| client preflight failures | `4` |
| client preflight rate limits | `4` |
| ctxhelm tool calls observed | `false` |
| outcome claim | `insufficient_comparable_lanes` |

Per-task preflight status:

| Task ID | Status | Outcome claim | Preflight failure |
| --- | --- | --- | --- |
| `memory-native-read` | `skipped` | `insufficient_comparable_lanes` | `rate_limited` |
| `semantic-contribution` | `skipped` | `insufficient_comparable_lanes` | `rate_limited` |
| `graph-edge-budget` | `skipped` | `insufficient_comparable_lanes` | `rate_limited` |
| `governor-release-proof` | `skipped` | `insufficient_comparable_lanes` | `rate_limited` |

## Decision

Do not claim fresh Claude retrieval-quality lift. The current blocker is client
availability, not a measured ctxhelm retrieval failure. Codex remains available
for outcome-matrix work, but the specific fresh Claude paired outcome gap stays
open until Claude Code preflight is no longer rate-limited.
