# Phase 316: Codex Retry-Cost Accounting

## Goal

Make Codex target-consumption retry measurable, not just successful. Phase 311
proved retry-enabled ctxhelm lanes could close evidence-only target gaps. This
phase adds source-free read-cost accounting so future R&D can tell whether that
retry improved consumption at acceptable overhead.

## Change

- `scripts/e2e-agent-run-codex.sh` now adds per-lane `retry` metadata for
  ctxhelm lanes.
- Triggered retries record before/after read-file count, irrelevant-read count,
  target-read coverage, and evidence-only target count.
- Single-task reports add `comparison.retryCost`.
- Suite reports add `aggregate.retryCost`.
- `ctxhelm eval agent-run` renders retry-cost summaries and per-lane retry
  deltas.

## Source-Free Boundary

The new fields are numeric counters, booleans, lane labels, and coverage
metrics. They do not store source text, raw prompts, transcripts, MCP traffic,
or raw command output.

## Validation

```bash
bash -n scripts/e2e-agent-run-codex.sh
cargo test -p ctxhelm --test cli_compat eval_agent_run --locked -- --nocapture
cargo test -p ctxhelm --test release_packaging codex_agent_run_e2e_script_contract --locked -- --nocapture
cargo test --workspace --locked
cargo run -p ctxhelm -- --help
```

All commands passed locally on 2026-06-08.

## Claim

Allowed claim:

```text
ctxhelm now records source-free retry read-cost metrics for Codex agent-run
reports.
```

Not yet allowed:

```text
ctxhelm retry improves efficiency.
```

That requires a fresh real Codex suite with the new `retryCost` fields.
