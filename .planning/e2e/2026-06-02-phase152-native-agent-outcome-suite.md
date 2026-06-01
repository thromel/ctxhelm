# Phase 152 - Native-Agent Outcome Suite

## Goal

Follow the R&D memo's highest-priority bet: make native-agent outcome proof
repeatable across task suites instead of one-off paired runs.

## Implementation

- Extended `scripts/e2e-agent-run.sh` with `--suite SUITE.json`.
  - Suite files can be an array of task objects or an object with `tasks`.
  - Each task requires `task`/`prompt` and `targetFiles`/`target_files`.
  - The harness runs the existing read-only Claude lanes for every task:
    native `baseline`, `ctxhelm-plan`, and `ctxhelm-brief`.
  - Per-task reports remain source-free and are embedded under `tasks`.
  - Aggregate suite metrics report task count, per-lane summaries, average
    target-coverage delta, total irrelevant-read delta, observed ctxhelm calls,
    and aggregate outcome claim.
- Updated `ctxhelm eval agent-run --report` to render suite aggregate metrics.
- Kept the existing `ctxhelm-agent-run-eval-v1` report contract additive and
  backward-compatible.
- Fixed the suite implementation to work on macOS Bash 3 by avoiding
  `mapfile`.

## Smoke Proof

Skipped-client suite contract smoke:

```bash
bash scripts/e2e-agent-run.sh \
  --suite "$TMPDIR/suite.json" \
  --repo . \
  --output "$TMPDIR/report.json"

cargo run -q -p ctxhelm --locked -- \
  eval agent-run --report "$TMPDIR/report.json"
```

Result:

```json
{"status":"skipped","tasks":2,"claim":"no_measured_lift"}
```

The smoke verified:

- `workflowKind = paired-agent-context-suite`
- `suite.taskCount = 2`
- `aggregate.taskCount = 2`
- `suite.rawTasksStored = false`
- Markdown output includes `## Suite Aggregate` and `## Suite Lanes`

## Validation

```bash
bash -n scripts/e2e-agent-run.sh scripts/smoke-public-real-clients.sh scripts/verify-public-archive-install.sh
cargo test -p ctxhelm --test cli_compat eval_agent_run --locked
cargo test -p ctxhelm --test release_packaging agent_run_e2e_script_contract --locked
```

All commands passed.

## Scope

This phase strengthens the benchmark harness. It does not yet claim new
real-client outcome lift across a multi-task suite. The next proof step is to
run a real Claude Code suite with `CTXHELM_RUN_REAL_CLIENT=1` against a fixed
task set, then compare aggregate native baseline versus ctxhelm-assisted lanes.

## Next R&D Step

Bet 2 from the R&D memo: replace the heuristic lexical scanner with a measured
Tantivy/BM25 plus exact symbol index prototype, gated by recall/latency/privacy
evidence.
