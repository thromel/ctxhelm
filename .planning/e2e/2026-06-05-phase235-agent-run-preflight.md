# Phase 235 - Agent Run Client Preflight

Date: 2026-06-05

## Goal

Continue the real-agent outcome-lift R&D lane after Phase 234 by retrying the
Claude Code five-lane matrix and making client availability failures cheaper
and less ambiguous.

## Root Cause

The fresh Claude Code `2.1.163` rerun still hit API status `429` before any
lane could make tool calls. The existing harness handled this as a source-free
rate-limit artifact, but it launched all five lanes before concluding client
availability was the blocker. It also reported missing required ctxhelm calls
for rate-limited lanes, which could be misread as guidance or compliance
failure even though Claude never had a chance to call tools.

## Changes

- `scripts/e2e-agent-run.sh` now runs a source-free Claude Code preflight before
  the lane matrix by default.
- The preflight is controlled by:
  - `CTXHELM_AGENT_RUN_PREFLIGHT`
  - `CTXHELM_AGENT_RUN_PREFLIGHT_TIMEOUT_SECONDS`
- Reports now include `clientPreflight`.
- Suite reports aggregate:
  - `clientPreflightCount`
  - `clientPreflightFailureCount`
  - `clientPreflightRateLimitCount`
- If preflight detects a client failure, lane execution short-circuits while
  still collecting ctxhelm evidence for assisted lanes.
- Rate-limited lanes now use `ctxhelmCallCompliance = "client_unavailable"`
  instead of counting missing required ctxhelm calls.

## Validation

Focused checks:

```bash
bash -n scripts/e2e-agent-run.sh
cargo test -p ctxhelm --test release_packaging agent_run_e2e_script_contract --locked
git diff --check
```

Fresh Claude Code attempt:

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=90 \
bash scripts/e2e-agent-run.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve paired agent-run lane matrix" \
  --target-file scripts/e2e-agent-run.sh \
  --target-file docs/feedback.md \
  --output .ctxhelm/e2e/phase235-agent-run-preflight.json
```

Renderer check:

```bash
target/debug/ctxhelm eval agent-run \
  --report .ctxhelm/e2e/phase235-agent-run-preflight.json \
  --format json
```

## Result

- `clientPreflight.status = failed`
- `clientPreflight.clientFailureKind = rate_limited`
- `clientPreflight.clientApiErrorStatus = 429`
- `comparison.rateLimitsObserved = true`
- `comparison.missingRequiredCtxhelmCallsObserved = false`
- `comparison.invalidRequiredCtxhelmCallsObserved = false`
- `comparison.ctxhelmEvidenceMissesObserved = false`
- `comparison.outcomeClaim = insufficient_comparable_lanes`
- `comparison.recommendedResearchActions = retry_real_client_when_available`
- `ctxhelm-plan`, `ctxhelm-brief`, `ctxhelm-standard`, and `ctxhelm-memory`
  each surface both expected targets in ctxhelm evidence.

## Interpretation

This is not outcome-lift proof because Claude Code remains rate-limited. It is
R&D hardening: the harness now proves client unavailability once, avoids
spending five full lane attempts under a known rate limit, avoids false missing
required-call signals, and confirms the ctxhelm evidence side of the selected
task still surfaces all expected targets.
