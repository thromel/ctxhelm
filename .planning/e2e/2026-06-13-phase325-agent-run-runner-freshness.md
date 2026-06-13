# Phase 325: Agent-Run Runner Freshness

## Goal

Prevent a saved real-agent outcome report from passing release validation after
the Codex agent-run harness changes.

Phase 323 made saved agent-run reports gateable. Phase 324 made the gate write a
source-free audit artifact. Phase 325 ties that saved proof to the current
runner script hash.

## Change

`scripts/check-agent-run-proof.py` now accepts:

```text
--current-runner-script scripts/e2e-agent-run-codex.sh
```

When provided, the checker computes the current runner script SHA-256 and
requires it to match `runner.scriptSha256` in the saved report.

The JSON audit artifact now records:

```text
runner.currentRunnerScriptName
runner.currentRunnerScriptSha256
runner.matchesCurrentRunnerScript
thresholds.currentRunnerScriptName
thresholds.requireCurrentRunnerScript
```

The release gate passes the current Codex runner path whenever
`CTXHELM_AGENT_RUN_PROOF_REPORT` is set. This keeps saved proof usable for
release validation only while the report's runner fingerprint still matches the
current harness.

## Validation

The committed Phase 322 report still passes because its runner fingerprint
matches the current runner script:

```text
runner.scriptSha256 = 52d3a4b3b6d57609f1e519db3d141cfa72bc4f97a8af4e8de2b63e015399deac
current script hash = 52d3a4b3b6d57609f1e519db3d141cfa72bc4f97a8af4e8de2b63e015399deac
```

The focused Rust contract now also creates a stale-runner fixture by replacing
`runner.scriptSha256` with a different SHA-256 value and proves the checker
rejects it.

```bash
cargo test -p ctxhelm --test release_packaging \
  agent_run_proof_checker_accepts_phase322_and_rejects_regression --locked
```
