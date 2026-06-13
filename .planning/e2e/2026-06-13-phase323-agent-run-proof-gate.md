# Phase 323: Agent-Run Proof Gate

## Goal

Make the Phase 322 real-Codex outcome proof machine-checkable without requiring
the release gate to rerun a live agent client.

## Change

Added:

```text
scripts/check-agent-run-proof.py
```

The checker validates source-free paired agent-run JSON reports. For suite
reports it verifies:

```text
schemaVersion = ctxhelm-agent-run-eval-v1
workflowKind = paired-agent-context-suite
status = passed
source-free privacy fields
runner fingerprint metadata
rawTasksStored = false
task/comparison/comparable-lane floors
ctxhelm lane target-read coverage floors
retry-cost fields
no client failures
no rate limits
no forbidden commands
no ctxhelm evidence misses
no evidence-only targets
no under-read targets
no missing/invalid required ctxhelm calls
bounded read-file delta
minimum irrelevant-read improvement
```

The release gate now has an optional saved-report hook:

```bash
CTXHELM_AGENT_RUN_PROOF_REPORT=.ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json \
  CTXHELM_REQUIRE_AGENT_RUN_PROOF=1 \
  bash scripts/release-gate.sh
```

The hook does not run Codex, Claude, Cursor, or OpenCode. It validates persisted
source-free process evidence and records `agentRunOutcomeProof` plus
`agentRunOutcomeProofRequired` in `release-proof-summary.json`.

## Phase 322 Gate

The current strict gate for the committed Phase 322 report is:

```bash
python3 scripts/check-agent-run-proof.py \
  .ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json \
  --workflow suite \
  --require-outcome ctxhelm_improved \
  --min-task-count 4 \
  --min-comparison-eligible 4 \
  --min-comparable-ctxhelm-lanes 16 \
  --min-ctxhelm-target-read-coverage 1.0 \
  --max-extra-read-delta 2 \
  --min-irrelevant-read-delta 2 \
  --require-retry-cost \
  --require-runner-fingerprint
```

This keeps the public claim honest:

```text
allowed: saved Phase 322 Codex suite is comparable, source-free, target-first,
and ctxhelm_improved under the recorded thresholds

not allowed: all future agent clients, prompts, or task suites inherit the
Phase 322 claim without fresh report evidence
```

## Validation

```bash
chmod +x scripts/check-agent-run-proof.py
python3 -m py_compile scripts/check-agent-run-proof.py

python3 scripts/check-agent-run-proof.py \
  .ctxhelm/e2e/phase322-agent-run-codex-target-first-breadth-suite.json \
  --workflow suite \
  --require-outcome ctxhelm_improved \
  --min-task-count 4 \
  --min-comparison-eligible 4 \
  --min-comparable-ctxhelm-lanes 16 \
  --min-ctxhelm-target-read-coverage 1.0 \
  --max-extra-read-delta 2 \
  --min-irrelevant-read-delta 2 \
  --require-retry-cost \
  --require-runner-fingerprint

cargo test -p ctxhelm --test release_packaging \
  agent_run_proof_checker_accepts_phase322_and_rejects_regression --locked
```
