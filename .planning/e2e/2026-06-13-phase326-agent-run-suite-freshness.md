# Phase 326: Agent-Run Suite Freshness

## Goal

Prevent a saved real-agent outcome report from passing release validation after
the task suite changes.

Phase 325 tied saved proof to the current Codex runner script. Phase 326 ties
the same saved proof to the current four-task R&D breadth suite.

## Change

`scripts/check-agent-run-proof.py` now accepts:

```text
--current-suite .planning/e2e/2026-06-06-phase251-codex-rd-suite.json
```

When provided, the checker computes the current suite file SHA-256 and requires
it to match `suite.suiteSha256` in the saved suite report.

The JSON audit artifact now records:

```text
suite.currentSuiteName
suite.currentSuiteSha256
suite.matchesCurrentSuite
thresholds.currentSuiteName
thresholds.requireCurrentSuite
```

The release gate passes the current R&D breadth suite path whenever
`CTXHELM_AGENT_RUN_PROOF_REPORT` is set. This keeps saved proof usable only
while both the runner and task suite match the current checked-in proof
contract.

## Validation

The committed Phase 322 report still passes because its suite fingerprint
matches the current suite file:

```text
suite.suiteSha256 = d04f86b3b8fb792a6d8dad7b493f728b1b78901a63a473dd004f2247b2b54afe
current suite hash = d04f86b3b8fb792a6d8dad7b493f728b1b78901a63a473dd004f2247b2b54afe
```

The focused Rust contract now creates a stale-suite fixture by replacing
`suite.suiteSha256` with a different SHA-256 value and proves the checker
rejects it.

```bash
cargo test -p ctxhelm --test release_packaging \
  agent_run_proof_checker_accepts_phase322_and_rejects_regression --locked
```
