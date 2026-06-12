# Phase 317: Codex Retry-Cost Proof And Suite Resume

## Goal

Turn Phase 316 retry-cost instrumentation into real Codex evidence, then fix
the harness gap exposed by the first full-suite rerun attempt.

## Full-Suite Attempt

A fresh real Codex breadth-suite rerun was attempted with:

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
bash scripts/e2e-agent-run-codex.sh \
  --repo "$PWD" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --output .ctxhelm/e2e/phase317-agent-run-codex-rd-breadth-suite-retry-cost.json
```

The external runner timed out after 20 minutes before the suite aggregate was
written. No final report was produced. This is not negative retrieval evidence;
it is a harness durability gap. The old suite path stored per-task reports only
inside a temporary directory and emitted the aggregate only after every task
completed, so a wall-clock timeout could lose completed source-free task proof.

## Focused Real Codex Proof

The focused governor/release-proof task completed successfully:

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
CTXHELM_RUN_REAL_CLIENT=1 \
CTXHELM_AGENT_RUN_TIMEOUT_SECONDS=150 \
bash scripts/e2e-agent-run-codex.sh \
  --repo "$PWD" \
  --task "Identify the files relevant to improving ctxhelm context governor decision reports and release-gate proof. Do not edit files." \
  --target-file crates/ctxhelm/src/main.rs \
  --target-file crates/ctxhelm-core/src/contracts.rs \
  --target-file scripts/smoke-governor.sh \
  --target-file docs/context-governor.md \
  --output .ctxhelm/e2e/phase317-agent-run-codex-governor-retry-cost.json
```

Summary:

```text
status: passed
outcomeClaim: ctxhelm_improved
comparisonEligible: true
comparableCtxhelmLaneCount: 4
targetCoverageDelta: 0.0
targetReadCoverageDelta: +0.25
irrelevantReadDelta: +1
readFileDelta: 0
forbiddenCommandsObserved: false
clientFailuresObserved: false
rateLimitsObserved: false
ctxhelmEvidenceMissesObserved: false
ctxhelmEvidenceOnlyTargetsObserved: false
ctxhelmUnderReadTargetsObserved: false
```

Retry cost:

```text
retryTriggeredLanes: 4
retrySelectedLanes: 4
avgReadFilesBeforeRetry: 5.0
avgReadFilesAfterRetry: 6.5
avgIrrelevantReadsBeforeRetry: 2.5
avgIrrelevantReadsAfterRetry: 2.5
targetReadCoverageBeforeRetry: 0.625
targetReadCoverageAfterRetry: 1.0
evidenceOnlyTargetsBeforeRetry: 6
evidenceOnlyTargetsAfterRetry: 0
```

The best lane was `ctxhelm-memory`: target-read coverage `1.00`, read files
`5`, irrelevant reads `1`, retry target-read delta `+0.50`, retry read-file
delta `0`, and retry irrelevant-read delta `-2`.

## Harness Change

`scripts/e2e-agent-run-codex.sh --suite` now supports durable per-task
checkpointing:

```bash
bash scripts/e2e-agent-run-codex.sh \
  --repo "$PWD" \
  --suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json \
  --suite-work-dir .ctxhelm/e2e/phase317-suite-checkpoints \
  --output .ctxhelm/e2e/phase317-agent-run-codex-rd-breadth-suite-retry-cost.json
```

The same behavior is available through:

```bash
CTXHELM_AGENT_RUN_SUITE_WORK_DIR=.ctxhelm/e2e/phase317-suite-checkpoints
```

Completed per-task reports are written to the checkpoint directory and reused
on rerun. Reused reports must satisfy the source-free report contract:
`schemaVersion = ctxhelm-agent-run-eval-v1`, `workflowKind =
paired-agent-context-run`, non-empty lanes, `rawTaskStored = false`, and false
privacy flags for source text, raw prompts, transcripts, MCP traffic, and raw
command output.

Suite aggregates now include:

```json
{
  "suite": {
    "checkpointEnabled": true,
    "checkpointDirSha256": "...",
    "reusedTaskCount": 4
  },
  "tasks": [
    {
      "taskId": "example",
      "reusedCheckpoint": true
    }
  ]
}
```

## Source-Free Boundary

The checkpoint files are the same source-free per-task reports already consumed
by the suite aggregate. They do not store raw task text, prompts, model
transcripts, MCP traffic, raw command output, or source text. The aggregate
stores a checkpoint-directory hash, not the local path.

## Validation

```bash
bash -n scripts/e2e-agent-run-codex.sh
cargo test -p ctxhelm --test release_packaging codex_agent_run_e2e_script_contract --locked -- --nocapture
```

The skipped-client suite smoke also verified checkpoint behavior: first run
wrote 4 task reports with `reusedTaskCount = 0`; second run reused all 4 reports
with `reusedTaskCount = 4`, `rawTasksStored = false`, and `sourceTextLogged =
false`.

## Claim

Allowed:

```text
ctxhelm has real Codex retry-cost proof for one focused governor/release-proof
task, and retry closed 6 evidence-only target gaps across 4 ctxhelm lanes while
raising average target-read coverage from 0.625 to 1.0.
```

Allowed:

```text
The Codex suite runner can now checkpoint and resume source-free per-task
reports, making long real-client suites more robust to wall-clock interruption.
```

Not allowed yet:

```text
ctxhelm retry improves broad-suite efficiency.
```

The full four-task real Codex breadth suite still needs to be rerun with
checkpointing enabled. The focused proof increased average reads from `5.0` to
`6.5` while holding average irrelevant reads flat at `2.5`.

