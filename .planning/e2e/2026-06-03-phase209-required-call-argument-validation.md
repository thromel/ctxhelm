# Phase 209 - Agent-Run Required Call Argument Validation

## Goal

Prevent paired real-agent R&D reports from treating a ctxhelm-assisted lane as
comparable when the agent called a required ctxhelm tool with missing or invalid
arguments.

## Change

- Required ctxhelm calls are now represented as source-free specs, not just tool
  names.
- `ctxhelm-plan` requires a valid `prepare_task` call with:
  - explicit repo;
  - task argument.
- `ctxhelm-brief` requires:
  - valid `prepare_task` with explicit repo and task;
  - valid `get_pack` with explicit repo, task, `budget = "brief"`,
    `format = "json"`, and `recordTrace = false`.
- Lane reports now expose:
  - `requiredCtxhelmCallSpecs`;
  - `invalidRequiredCtxhelmCalls`;
  - `invalidRequiredCtxhelmCallCount`;
  - `ctxhelmCallCompliance = invalid` when a required tool name appeared but
    the sanitized arguments did not satisfy the spec.
- Single-run and suite reports now expose:
  - `invalidRequiredCtxhelmCallsObserved`;
  - invalid required-call counts in lane summaries.
- `ctxhelm eval agent-run` renders invalid required-call reasons such as
  `prepare_task[repo, task; attempts=1]`.

## Validation

- `bash -n scripts/e2e-agent-run.sh` passed.
- `cargo test -p ctxhelm eval_agent_run --locked -- --nocapture` passed with a
  fixture that renders invalid required-call reasons.
- `cargo test -p ctxhelm agent_run_e2e_script_contract --locked -- --nocapture`
  passed.
- Skipped smoke report `/tmp/ctxhelm-phase209-skipped.json` passed with missing
  required calls and zero invalid-call false positives.
- Real Claude Code probe `/tmp/ctxhelm-phase209-real.json` still hit the known
  rate limit and reported `clientFailureKind = rate_limited`,
  `clientApiErrorStatus = 429`, `rateLimitObserved = true`, and
  `invalidRequiredCtxhelmCallsObserved = false`.
- `cargo run -p ctxhelm --locked -- --help` passed.
- `cargo test --workspace --locked --no-fail-fast` passed.
- `CTXHELM_ALLOW_DIRTY=1 bash scripts/release-gate.sh` passed.

## Interpretation

This phase does not claim a new ctxhelm retrieval or consumption lift. It
improves proof quality: a wrong-repo, missing-task, wrong-budget, wrong-format,
or trace-recording `get_pack` call can no longer satisfy the ctxhelm-assisted
lane contract.
