# Phase 322: Codex Target-First Breadth Proof

## Goal

Turn Phase 321's focused target-first prompt improvement into broad Codex
agent-run evidence on the four-task R&D breadth suite.

The acceptance gate was:

```text
preserve 1.0 ctxhelm target-read coverage
reduce noisy plan/brief/standard reads versus Phase 318
keep evidence-only targets closed after retry
avoid under-read targets
avoid forbidden commands, client failures, and rate limits
```

## Final Artifact

```text
.ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json
```

The run used:

```text
Codex CLI: codex-cli 0.137.0
Suite: .planning/e2e/2026-06-06-phase251-codex-rd-suite.json
Suite checkpointing: enabled
Reused checkpoints in final run: 0
Runner fingerprint: runner_fingerprint_v1
```

## Result

```text
status: passed
outcome: ctxhelm_improved
task count: 4
comparison-eligible tasks: 4
comparable ctxhelm lanes: 16
target-read coverage delta average: +0.3125
target coverage delta average: +0.25
forbidden commands: false
missing required ctxhelm calls: false
invalid required ctxhelm calls: false
client failures: false
rate limits: false
ctxhelm evidence misses: false
ctxhelm evidence-only targets: false
ctxhelm under-read targets: false
```

## Lane Metrics

```text
baseline:
  target-read coverage: 0.6875
  reads: 17
  irrelevant reads: 8
  read precision: 0.5294
  irrelevant-read rate: 0.4706

ctxhelm-plan:
  target-read coverage: 1.0
  reads: 23
  irrelevant reads: 10
  read precision: 0.5652
  irrelevant-read rate: 0.4348

ctxhelm-brief:
  target-read coverage: 1.0
  reads: 22
  irrelevant reads: 9
  read precision: 0.5909
  irrelevant-read rate: 0.4091

ctxhelm-standard:
  target-read coverage: 1.0
  reads: 23
  irrelevant reads: 10
  read precision: 0.5652
  irrelevant-read rate: 0.4348

ctxhelm-memory:
  target-read coverage: 1.0
  reads: 21
  irrelevant reads: 8
  read precision: 0.6190
  irrelevant-read rate: 0.3810
```

## Comparison To Phase 318

Phase 318 had already proven target-read reliability, but plan/brief/standard
were noisy:

```text
ctxhelm-plan:     36 reads, 23 irrelevant, precision 0.3611
ctxhelm-brief:    35 reads, 22 irrelevant, precision 0.3714
ctxhelm-standard: 35 reads, 22 irrelevant, precision 0.3714
ctxhelm-memory:   22 reads,  9 irrelevant, precision 0.5909
```

Phase 322 preserves `1.0` target-read coverage in every ctxhelm lane and cuts
the noisy lanes materially:

```text
ctxhelm-plan:     36 -> 23 reads, 23 -> 10 irrelevant
ctxhelm-brief:    35 -> 22 reads, 22 ->  9 irrelevant
ctxhelm-standard: 35 -> 23 reads, 22 -> 10 irrelevant
ctxhelm-memory:   22 -> 21 reads,  9 ->  8 irrelevant
```

During the Phase 322 rerun, an intermediate semantic-contribution checkpoint
showed the memory lane reading docs/planning evidence before all returned target
files. The memory prompt now makes the contract explicit: memory evidence may
prioritize target files, but it must not displace `targetFiles` or cause
selectedMemory/docs/planning paths to be read before returned targets unless
those paths are themselves targets.

## Retry Cost

```text
retry triggered lanes: 5
retry selected lanes: 5
avg reads before retry: 5.4
avg reads after retry: 5.8
avg irrelevant before retry: 3.0
avg irrelevant after retry: 2.4
target-read coverage before retry: 0.7
target-read coverage after retry: 1.0
evidence-only targets before retry: 5
evidence-only targets after retry: 0
```

Retry still matters: target-first guidance reduces noise, but retry remains the
mechanism that closes evidence-only target consumption when Codex initially
skips surfaced files.

## Claim

Allowed:

```text
On the four-task real Codex breadth suite, ctxhelm target-first guidance
preserved 1.0 target-read coverage in all ctxhelm lanes while materially
reducing noisy plan/brief/standard reads versus Phase 318.
```

Allowed:

```text
The efficient Phase 322 lane is ctxhelm-memory: it recovers 4 target reads over
baseline, reaches 1.0 target-read coverage, improves read precision from 0.5294
to 0.6190, lowers irrelevant-read rate from 0.4706 to 0.3810, and adds no extra
irrelevant reads.
```

Not allowed:

```text
ctxhelm always reduces absolute reads versus a native baseline.
```

The suite still reports `readFileDeltaSum = -2`, meaning ctxhelm performs two
more total reads across the best per-task lanes while recovering missed targets
and reducing irrelevant reads by two.

## Validation

```bash
CTXHELM_RUN_REAL_CLIENT=1 CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=240 \
  bash scripts/e2e-agent-run-codex.sh \
  --repo . \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --suite-work-dir .ctxhelm/e2e/phase322-codex-target-first-checkpoints \
  --output .ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json

cargo run -q -p ctxhelm -- eval agent-run \
  --report .ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json
```
