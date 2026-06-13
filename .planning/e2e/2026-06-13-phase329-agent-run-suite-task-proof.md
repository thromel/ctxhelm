# Phase 329: Agent-Run Suite Task Proof

## Goal

Prevent a saved real-agent outcome report from passing release validation when
its top-level `suite.suiteSha256` matches the current suite, but the embedded
`tasks[*]` entries no longer match that suite.

Phase 326 tied saved proof to the current suite file hash. Phase 328 validated
every nested task lane. Phase 329 connects those two checks by validating the
saved task hashes and target lists against the current suite task definitions.

## Change

When `--current-suite` is provided, `scripts/check-agent-run-proof.py` now
re-parses the current suite file and validates:

```text
len(report.tasks) == len(currentSuite.tasks)
tasks[*].taskSha256 == sha256(currentSuite.tasks[*].task)
tasks[*].targetFiles == currentSuite.tasks[*].targetFiles
```

The validation uses the same source-free task hash contract as
`scripts/e2e-agent-run-codex.sh`: SHA-256 of the task text, without storing raw
task text in the saved report.

The JSON audit artifact now records:

```text
suiteTaskChecks.strictCurrentSuiteTaskChecks
suiteTaskChecks.reportTaskCount
suiteTaskChecks.currentSuiteTaskCount
suiteTaskChecks.matchesCurrentSuiteTasks
```

## Validation

The committed Phase 322 report still passes with:

```text
suiteTaskChecks.reportTaskCount = 4
suiteTaskChecks.currentSuiteTaskCount = 4
suiteTaskChecks.matchesCurrentSuiteTasks = true
```

The focused Rust contract now creates a stale-suite-task fixture by changing
`tasks[0].targetFiles[0]` while leaving `suite.suiteSha256` and aggregate fields
unchanged. The checker rejects that fixture.

```bash
cargo test -p ctxhelm --test release_packaging \
  agent_run_proof_checker_accepts_phase322_and_rejects_regression --locked
```
