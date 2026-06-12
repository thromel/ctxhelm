# Phase 320: Codex Checkpoint Fingerprints

## Goal

Make the Codex breadth-suite checkpoint cache safe for the next read-efficiency
optimization round. Phase 317 made long suites resumable, but checkpoint reuse
only validated the basic report/privacy contract. That was enough for timeout
recovery, but not enough for prompt or runner R&D: a changed runner could reuse
old per-task reports and make a new prompt look better or worse without fresh
evidence.

## Change

`scripts/e2e-agent-run-codex.sh` now records source-free runner metadata in both
single-task and suite reports:

```json
{
  "runner": {
    "name": "e2e-agent-run-codex",
    "contractVersion": "ctxhelm-agent-run-codex-runner-v1",
    "scriptSha256": "...",
    "checkpointValidation": "runner_fingerprint_v1"
  }
}
```

Suite checkpoint reuse now requires the checkpointed per-task report to match:

```text
schemaVersion
workflowKind
runner name
runner contractVersion
runner scriptSha256
runner checkpointValidation
ctxhelmVersion
Codex client version
task hash
targetFiles list
privacy source-free contract
non-empty lanes
```

If any of those values drift, the stale checkpoint is deleted and the task is
rerun.

## Why This Matters

The next product-improvement gate is read efficiency: preserve the Phase 318
`1.0` ctxhelm target-read coverage while reducing noisy plan/brief/standard
reads. That work will likely change prompts or lane ordering. Without runner
fingerprints, old checkpoints could mask those changes. With fingerprints, every
prompt/runner edit invalidates stale task reports by construction.

## Claim

Allowed:

```text
Codex suite checkpoints are now source-free and runner-fingerprinted, so stale
per-task reports are not reused after runner, task, target, ctxhelm-version, or
client-version drift.
```

Not allowed:

```text
The prompt-efficiency change itself is proven.
```

This phase prepares the evidence harness for that change; it does not tune the
agent prompts yet.

## Validation

```bash
bash -n scripts/e2e-agent-run-codex.sh

# Skipped-client checkpoint smoke:
# 1. Run a two-task suite with --suite-work-dir.
# 2. Rerun the same suite and verify reusedTaskCount == taskCount.
# 3. Mutate one checkpoint runner.scriptSha256.
# 4. Rerun and verify only the unmutated checkpoint is reused.

cargo test -p ctxhelm --test release_packaging codex_agent_run_e2e_script_contract --locked -- --nocapture
cargo test --workspace --locked
```
