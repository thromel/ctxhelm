# Phase 312 - Claude Cross-Agent Refresh

## Scope

Phase 311 closed the measured Codex evidence-only target-consumption gap. Claude
Opus was then asked for a source-free second opinion on the next R&D step. Its
recommendation was to prioritize cross-agent replication of the Codex Phase 311
breadth suite, stop chasing the weak semantic retention separator, record a
completion gate, and watch retry overfitting/read-cost.

A tiny manual Claude Opus probe returned `OK`, so this phase refreshed the
machine-checkable Claude availability and then attempted the same four-task R&D
breadth suite with preflight disabled. The purpose was to distinguish "small
prompt works" from "full source-free paired outcome proof is comparable."

## Availability Refresh

Command:

```bash
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
CTXHELM_AGENT_AVAILABILITY_TIMEOUT_SECONDS=60 \
bash scripts/e2e-agent-client-availability.sh \
  --repo "$(pwd)" \
  --task "Refresh Claude post-Phase311 cross-agent outcome proof" \
  --output .ctxhelm/e2e/phase312-agent-client-availability.json
```

Result:

| Metric | Value |
| --- | ---: |
| status | `passed` |
| schema | `ctxhelm-agent-client-availability-v1` |
| Claude Code version | `2.1.163 (Claude Code)` |
| Claude status | `unavailable` |
| Claude client failure | `rate_limited` |
| Claude rate limit observed | `true` |
| Claude client exit status | `0` |
| Codex CLI version | `codex-cli 0.137.0` |
| Codex status | `available` |
| ready clients | `1` |
| rate-limited clients | `1` |

The availability artifact remains source-free and local-only. It records
deterministic explicit-repo Codex MCP evidence, while Claude emits a
rate-limit event despite exit status `0`.

## Claude Breadth-Suite Attempt

Command:

```bash
CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_PREFLIGHT=0 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
CTXHELM_AGENT_RUN_PREFLIGHT_TIMEOUT_SECONDS=60 \
bash scripts/e2e-agent-run.sh \
  --repo "$(pwd)" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --output .ctxhelm/e2e/phase312-agent-run-claude-rd-breadth-suite-preflight-disabled.json
```

Aggregate:

| Metric | Value |
| --- | ---: |
| status | `degraded` |
| schema | `ctxhelm-agent-run-eval-v1` |
| client | `claude 2.1.163 (Claude Code)` |
| tasks | `4` |
| comparison-eligible tasks | `0` |
| comparable ctxhelm lanes | `0` |
| outcome claim | `insufficient_comparable_lanes` |
| ctxhelm tool calls observed | `true` |
| client failures observed | `true` |
| rate limits observed | `true` |
| evidence misses / evidence-only / under-read | `false` / `false` / `false` |
| missing / invalid required ctxhelm calls | `false` / `false` |
| forbidden tool calls observed | `true` |
| recommended action | `retry_real_client_when_available(p1)` |

Lane summary:

| Lane | Passed | Eligible | Avg target-read coverage | Client failures | Rate limits | Forbidden calls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | `4` | `0` | `0.5208333333333333` | `4` | `4` | `0` |
| ctxhelm-plan | `3` | `0` | `0.5833333333333333` | `4` | `4` | `0` |
| ctxhelm-brief | `3` | `0` | `0.25` | `4` | `4` | `0` |
| ctxhelm-standard | `3` | `0` | `0.5` | `4` | `4` | `0` |
| ctxhelm-memory | `1` | `0` | `0.0` | `4` | `4` | `12` |

Two `ctxhelm-memory` task attempts recorded forbidden `Bash` tool calls. This is
useful boundary evidence, but it is not a retrieval-quality comparison because
every task and every lane also records `clientFailureKind = rate_limited` and
`evaluationEligible = false`.

## Decision

Phase 312 does not replicate the Codex Phase 311 outcome proof on Claude. It
confirms that manual tiny-prompt availability is not enough for a comparable
source-free paired suite: the actual Claude suite remains rate-limited and
therefore correctly reports zero comparable lanes.

Keep the cross-agent gap scoped as external client availability plus
read-only-boundary evidence, not a ctxhelm retrieval failure. The next valid
Claude outcome step is to retry the same suite when the client no longer emits
rate-limit events.
