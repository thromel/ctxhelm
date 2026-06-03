# Phase 208 - Agent-Run Client Failure Classification

## Goal

Keep real-client availability failures separate from ctxhelm retrieval, pack, and
agent-consumption behavior in paired agent-run R&D reports.

## Trigger

A Phase 207 real Claude Code probe exited before any lane could read files or
call ctxhelm. A direct minimal Claude Code run showed the cause was a session
rate limit with API status `429`, not a ctxhelm MCP, retrieval, pack, or report
parsing failure.

## Change

- `scripts/e2e-agent-run.sh` now classifies source-free client failures per
  lane through:
  - `clientFailureKind`
  - `clientApiErrorStatus`
  - `rateLimitObserved`
- Recognized failure kinds include:
  - `rate_limited`
  - `api_error`
  - `client_error`
  - `timeout`
  - `client_exit_nonzero`
- Single-run comparisons expose:
  - `clientFailuresObserved`
  - `rateLimitsObserved`
- Suite reports aggregate:
  - client-failure observations;
  - rate-limit observations;
  - per-lane client failure and rate-limit counts.
- `ctxhelm eval agent-run` renders those fields in single-run and suite markdown
  reports.

## Validation

- Phase 207 skipped-report smoke showed missing required calls and
  `insufficient_comparable_lanes` without raw source/prompt storage.
- A real Claude Code probe wrote `/tmp/ctxhelm-phase208-real.json` and classified
  all three lanes as `clientFailureKind = rate_limited`,
  `clientApiErrorStatus = 429`, and `rateLimitObserved = true`.
- Rendered report output showed `Client failures observed: true`,
  `Rate limits observed: true`, and `Comparison eligible: false`.
- The report still stored only path labels, metrics, request hashes, and
  source-free failure metadata; it did not store raw Claude output.
- `cargo run -p ctxhelm --locked -- --help` passed.
- `cargo test --workspace --locked --no-fail-fast` passed.
- `CTXHELM_ALLOW_DIRTY=1 bash scripts/release-gate.sh` passed.

## Interpretation

This phase does not prove ctxhelm lift. It prevents future R&D runs from
misclassifying client unavailability as weak ctxhelm context. Real-client outcome
proof should be rerun after Claude Code quota resets.
