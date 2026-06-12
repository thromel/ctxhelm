# Phase 319: Agent Read-Efficiency Metrics

## Goal

Make Phase 318's read-overhead finding actionable. The Codex breadth suite
proved target-consumption reliability, but the report only said
`optimize_agent_read_efficiency`. This phase adds source-free efficiency
metrics that identify which ctxhelm lane is most efficient and how much extra
read cost was paid for recovered target reads.

## Change

`scripts/e2e-agent-run-codex.sh` now records per-lane:

```text
targetReadPrecision
irrelevantReadRate
readsPerTargetRead
```

Single-task and suite reports now include:

```json
{
  "readEfficiency": {
    "baselineLane": "baseline",
    "efficientCtxhelmLane": "ctxhelm-memory",
    "baselineTargetReadCoverage": 0.625,
    "efficientTargetReadCoverage": 1.0,
    "targetReadCoverageDelta": 0.375,
    "baselineReadFileCount": 15,
    "efficientReadFileCount": 22,
    "extraReadFileCount": 7,
    "baselineIrrelevantReadCount": 7,
    "efficientIrrelevantReadCount": 9,
    "extraIrrelevantReadCount": 2,
    "baselineTargetReadPrecision": 0.5333333333333333,
    "efficientTargetReadPrecision": 0.5909090909090909,
    "targetReadPrecisionDelta": 0.05757575757575761,
    "baselineIrrelevantReadRate": 0.4666666666666667,
    "efficientIrrelevantReadRate": 0.4090909090909091,
    "irrelevantReadRateDelta": -0.05757575757575756,
    "recoveredTargetReadCount": 5,
    "extraReadsPerRecoveredTarget": 1.4,
    "extraIrrelevantReadsPerRecoveredTarget": 0.4
  }
}
```

`ctxhelm eval agent-run` renders the new fields in source-free markdown.

## Phase 318 Report Re-Render

The Phase 318 aggregate was regenerated from the existing source-free
checkpoints. No real Codex tasks were rerun. The refreshed report shows:

```text
baseline:
  target-read coverage: 0.625
  read precision: 0.5333333333333333
  irrelevant-read rate: 0.4666666666666667
  reads per target read: 1.875

ctxhelm-memory:
  target-read coverage: 1.0
  read precision: 0.5909090909090909
  irrelevant-read rate: 0.4090909090909091
  reads per target read: 1.6923076923076923
  extra reads: 7
  extra irrelevant reads: 2
  recovered target reads: 5
  extra reads per recovered target: 1.4
  extra irrelevant reads per recovered target: 0.4
```

This reframes the Phase 318 efficiency story. The efficient ctxhelm lane has
better precision and lower irrelevant-read rate than baseline, but still uses
more absolute reads because it recovers five target reads the baseline missed.

The noisy lanes are now explicit:

```text
ctxhelm-plan:
  read precision: 0.3611111111111111
  irrelevant-read rate: 0.6388888888888888
  reads per target read: 2.769230769230769

ctxhelm-brief:
  read precision: 0.37142857142857144
  irrelevant-read rate: 0.6285714285714286
  reads per target read: 2.6923076923076925

ctxhelm-standard:
  read precision: 0.37142857142857144
  irrelevant-read rate: 0.6285714285714286
  reads per target read: 2.6923076923076925
```

## Claim

Allowed:

```text
ctxhelm now reports source-free read-efficiency metrics for Codex agent-run
reports, including read precision, irrelevant-read rate, efficient ctxhelm lane,
and extra reads per recovered target.
```

Allowed:

```text
In the Phase 318 Codex breadth suite, `ctxhelm-memory` was the efficient ctxhelm
lane: it reached 1.0 target-read coverage with better precision and lower
irrelevant-read rate than baseline, at the cost of 7 extra reads and 2 extra
irrelevant reads to recover 5 target reads.
```

Not allowed:

```text
ctxhelm broadly reduces absolute read count.
```

The next implementation slice should use these fields as the acceptance gate:
preserve 1.0 target-read coverage while reducing extra reads per recovered
target and moving plan/brief/standard closer to memory-lane precision.

## Validation

```bash
bash -n scripts/e2e-agent-run-codex.sh
cargo test -p ctxhelm --test cli_compat eval_agent_run --locked -- --nocapture
cargo test -p ctxhelm --test release_packaging codex_agent_run_e2e_script_contract --locked -- --nocapture
```

