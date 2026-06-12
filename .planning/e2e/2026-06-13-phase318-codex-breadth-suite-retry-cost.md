# Phase 318: Codex Breadth-Suite Retry-Cost Proof

## Goal

Complete the Phase 315/316 retry-cost loop with a fresh real Codex breadth
suite, using Phase 317 checkpointing so long real-client work survives
interruption.

## Run

The suite was run with durable source-free checkpoints:

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
bash scripts/e2e-agent-run-codex.sh \
  --repo "$PWD" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --suite-work-dir .ctxhelm/e2e/phase318-codex-rd-breadth-checkpoints \
  --output .ctxhelm/e2e/phase318-agent-run-codex-rd-breadth-suite-retry-cost.json
```

The first full run completed all four tasks and wrote four checkpoint reports.
The aggregate was regenerated once from checkpoints after report-action logic
was tightened; the regenerated aggregate reports `reusedTaskCount = 4`.

## Aggregate Result

```text
status: passed
workflowKind: paired-agent-context-suite
client: codex-cli 0.137.0
taskCount: 4
comparisonEligibleCount: 4
comparableCtxhelmLaneCount: 16
outcomeClaim: ctxhelm_improved
targetCoverageDeltaAverage: +0.375
targetReadCoverageDeltaAverage: +0.375
irrelevantReadDeltaSum: -2
readFileDeltaSum: -7
commandExecutionDeltaSum: +11
forbiddenCommandsObserved: false
clientFailuresObserved: false
rateLimitsObserved: false
missingRequiredCtxhelmCallsObserved: false
invalidRequiredCtxhelmCallsObserved: false
ctxhelmEvidenceMissesObserved: false
ctxhelmEvidenceOnlyTargetsObserved: false
ctxhelmUnderReadTargetsObserved: false
```

Privacy remained local/source-free:

```text
localOnly: true
sourceTextLogged: false
rawPromptStored: false
rawTranscriptStored: false
rawMcpTrafficStored: false
rawCommandOutputStored: false
remoteEmbeddingsUsed: false
remoteRerankingUsed: false
```

## Lane Summary

```text
baseline:
  average target-read coverage: 0.625
  target reads: 8
  missed targets: 5
  read files: 15
  irrelevant reads: 7

ctxhelm-plan:
  average target-read coverage: 1.0
  target reads: 13
  missed targets: 0
  read files: 36
  irrelevant reads: 23

ctxhelm-brief:
  average target-read coverage: 1.0
  target reads: 13
  missed targets: 0
  read files: 35
  irrelevant reads: 22

ctxhelm-standard:
  average target-read coverage: 1.0
  target reads: 13
  missed targets: 0
  read files: 35
  irrelevant reads: 22

ctxhelm-memory:
  average target-read coverage: 1.0
  target reads: 13
  missed targets: 0
  read files: 22
  irrelevant reads: 9
```

The memory lane is the efficient ctxhelm lane, but it still used more files and
irrelevant reads than the native baseline in this suite. The reliability gain is
clear; broad read-efficiency is not yet proven.

## Retry Cost

```text
retryTriggeredLanes: 4
retrySelectedLanes: 4
avgReadFilesBeforeRetry: 5.0
avgReadFilesAfterRetry: 6.75
avgIrrelevantReadsBeforeRetry: 3.25
avgIrrelevantReadsAfterRetry: 3.5
targetReadCoverageBeforeRetry: 0.5416666666666666
targetReadCoverageAfterRetry: 1.0
evidenceOnlyTargetsBeforeRetry: 6
evidenceOnlyTargetsAfterRetry: 0
```

Retry closed all evidence-only target gaps, but it increased reads and slightly
increased irrelevant reads. This makes retry a reliability mechanism, not an
efficiency win.

## Per-Task Results

```text
001 memory-native-read:
  outcome: ctxhelm_improved
  targetReadCoverageDelta: 0.0
  irrelevantReadDelta: 0
  readFileDelta: 0
  retryTriggeredLanes: 2

002 semantic-contribution:
  outcome: ctxhelm_improved
  targetReadCoverageDelta: +0.33333333333333337
  irrelevantReadDelta: -3
  readFileDelta: -4
  retryTriggeredLanes: 1

003 graph-edge-budget:
  outcome: ctxhelm_improved
  targetReadCoverageDelta: +0.6666666666666667
  irrelevantReadDelta: -1
  readFileDelta: -3
  retryTriggeredLanes: 0

004 governor-release-proof:
  outcome: ctxhelm_improved
  targetReadCoverageDelta: +0.5
  irrelevantReadDelta: +2
  readFileDelta: 0
  retryTriggeredLanes: 1
```

## Product Change

The Codex report action logic now emits:

```text
optimize_agent_read_efficiency(p2)
```

when ctxhelm improves target consumption but does so with extra reads or extra
irrelevant reads. This prevents a reliability-positive report from incorrectly
routing only to `preserve_current_agent_contract`.

## Claim

Allowed:

```text
On the four-task real Codex breadth suite, ctxhelm lanes reached 1.0 average
target-read coverage versus baseline 0.625, across 16 comparable ctxhelm lanes,
with no evidence misses, no evidence-only targets after retry, no under-read
targets, no forbidden commands, no client failures, and no rate limits.
```

Allowed:

```text
Codex retry enforcement closed 6 evidence-only target gaps and raised retry
target-read coverage from 0.542 to 1.0 in triggered lanes.
```

Not allowed:

```text
ctxhelm broadly reduces read overhead or irrelevant reads.
```

The next R&D slice should optimize agent read efficiency after target coverage
is secured. The current evidence points at lane ordering, target-first stop
rules, and memory-lane compaction rather than another retrieval algorithm.

## Validation

```bash
target/debug/ctxhelm eval agent-run \
  --report .ctxhelm/e2e/phase318-agent-run-codex-rd-breadth-suite-retry-cost.json

bash -n scripts/e2e-agent-run-codex.sh
cargo test -p ctxhelm --test release_packaging codex_agent_run_e2e_script_contract --locked -- --nocapture
```

