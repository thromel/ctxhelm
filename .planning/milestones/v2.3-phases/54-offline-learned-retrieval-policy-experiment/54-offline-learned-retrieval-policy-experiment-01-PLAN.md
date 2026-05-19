# Plan: Offline Learned Retrieval Policy Experiment

## Objective

Add an offline learned-policy proposal path that consumes source-free feature
exports plus feedback/outcome evidence, stores non-default profiles with
provenance, and blocks active application when thresholds fail.

## Tasks

1. Extend `RetrievalPolicyProfile` with schema version, training corpus ID,
   training sources, metric summary, baseline thresholds, and default
   eligibility.
2. Add an offline learner that reads local candidate feature exports,
   source-free feature labels, feedback quality reports, and outcome comparison
   reports.
3. Add guarded `apply` behavior so profiles with failed thresholds cannot
   become active.
4. Add `ctxpack eval policy learn` with threshold arguments and Markdown/JSON
   output.
5. Extend CLI tests for learned profile provenance, threshold pass/fail, apply,
   rollback, and source-free output.
6. Document the learned-policy workflow and update planning artifacts.

## Verification

- `cargo fmt --all`
- `CARGO_INCREMENTAL=0 cargo check --workspace`
- `CARGO_INCREMENTAL=0 cargo test -p ctxpack --test cli_compat eval_policy_and_outcome_reports_are_source_free`
- `CARGO_INCREMENTAL=0 cargo run -p ctxpack -- eval policy learn --help`
- `CARGO_INCREMENTAL=0 cargo run -p ctxpack -- --help`
- `CARGO_INCREMENTAL=0 cargo test --workspace`
- `git diff --check`
- `gsd-sdk query roadmap.analyze`

