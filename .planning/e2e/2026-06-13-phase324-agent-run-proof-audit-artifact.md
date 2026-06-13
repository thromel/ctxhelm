# Phase 324: Agent-Run Proof Audit Artifact

## Goal

Make the saved agent-run proof gate auditable after release validation, not just
pass/fail at command time.

Phase 323 added `scripts/check-agent-run-proof.py` and optional release-gate
validation through `CTXHELM_AGENT_RUN_PROOF_REPORT`. Phase 324 adds a
source-free machine-readable audit artifact so the release proof bundle records
which saved report and thresholds actually passed.

## Change

`scripts/check-agent-run-proof.py` now supports:

```text
--format text|json
--output PATH
```

JSON output uses:

```text
schemaVersion: ctxhelm-agent-run-proof-check-v1
```

The JSON artifact records:

```text
status
workflow
saved report filename
saved report SHA-256
thresholds
source-free privacy checks
runner fingerprint metadata
aggregate metrics
boundary checks
ctxhelm lane quality summaries
```

It does not store task text, raw prompts, transcripts, MCP traffic, command
output, source snippets, or absolute local report paths.

The release gate now writes:

```text
CTXHELM_PROOF_DIR/agent-run-outcome-proof.json
```

when `CTXHELM_AGENT_RUN_PROOF_REPORT` is set, and records:

```text
agentRunOutcomeProof
agentRunOutcomeProofRequired
agentRunOutcomeProofReport
```

inside `release-proof-summary.json`.

## Validation

```bash
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
  --require-runner-fingerprint \
  --format json \
  --output /tmp/ctxhelm-agent-run-proof-check.json

cargo test -p ctxhelm --test release_packaging \
  agent_run_proof_checker_accepts_phase322_and_rejects_regression --locked
```

The focused Rust contract verifies the committed Phase 322 report passes,
the JSON artifact exposes `ctxhelm-agent-run-proof-check-v1`, `reportSha256`,
thresholds, source-free privacy pass flags, and `ctxhelm_improved`, and a
corrupted evidence-only-target report fails.
