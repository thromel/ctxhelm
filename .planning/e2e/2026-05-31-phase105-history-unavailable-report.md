# Phase 105: History-Unavailable Embedded Reports

## Goal

Make benchmark and product-proof output robust when git history sampling is
unavailable, slow, or backed by offloaded objects. The Phase 104 four-repo
attempt showed the failure mode clearly: a RefactoringMiner `git rev-list`
timeout caused the repository report to become `null`, which made downstream
proof checking fail on shape rather than on an explicit quality verdict.

## Changes

- `evaluate_historical_commits` now degrades git history sampling failures to
  an embedded zero-commit historical eval report instead of returning an error.
- Benchmark repository reports keep `report: {...}` and set a source-free error
  message when no commits could be evaluated:

  ```text
  repository produced no evaluable commits; git history may be unavailable or timed out
  ```

- Product-proof corpus verdicts for that shape are
  `insufficient_evidence`, include the same source-free note, and block
  promotion honestly.
- Degraded zero-commit reports are not written to the historical eval cache, so
  a transient git timeout cannot poison later hydrated runs.

## Validation

Focused regression:

```bash
CARGO_TARGET_DIR=/tmp/ctxpack-phase105-target cargo test -p ctxpack-compiler benchmark_suite_embeds_report_when_history_sampling_is_unavailable -- --nocapture
```

CLI proof fixture:

```bash
CARGO_TARGET_DIR=/tmp/ctxpack-phase105-target cargo run -p ctxpack -- eval proof --config /tmp/ctxpack-phase105-historyless-config.json --format json > .ctxpack/e2e/phase105-history-unavailable-proof.json
```

Observed source-free proof summary:

```json
{
  "decision": "block",
  "repos": [
    {
      "name": "HistorylessFixture",
      "evaluatedCommits": 0,
      "hasReport": true,
      "error": "repository produced no evaluable commits; git history may be unavailable or timed out"
    }
  ],
  "verdict": "insufficient_evidence"
}
```

The product-proof checker now fails this proof for the correct reason:

```text
product proof releaseGate.decision was not promote: block
```

It no longer fails because an embedded benchmark repository report is missing.

## Impact

This does not make a broken or unhydrated large-history checkout count as a
passing proof. It makes the failure machine-checkable and source-free: agents,
release gates, and maintainers can distinguish "insufficient evidence because
history was unavailable" from schema corruption or missing report data.
