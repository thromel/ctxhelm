# Phase 121: CI Node 24 Runtime Guard

## Goal

Remove the known GitHub Actions Node.js 20 deprecation warning from the public
CI release gate before GitHub forces the migration later in 2026.

## Gap

The Phase 120 public CI workflow passed on `main`, but GitHub annotated both
jobs with a Node.js 20 actions runtime deprecation warning for `actions/checkout`
and `actions/cache`. The jobs were green, but the warning meant the release gate
was already carrying a time-bounded platform risk.

## Changes

- Added `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"` to the workflow-level
  environment in `.github/workflows/ci.yml`.
- Upgraded public CI JavaScript actions to their Node 24 major versions:
  `actions/checkout@v5` and `actions/cache@v5`.
- Added release-packaging contract coverage so the public CI workflow cannot
  drop the Node 24 opt-in or action major upgrades silently.
- Updated release docs and `scripts/check-release-docs.sh` so the CI runtime
  guard remains visible in the release surface.

## Validation

Passed locally:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p ctxhelm --test release_packaging ci_workflow_contract -- --nocapture
bash scripts/check-release-docs.sh
git diff --check
```

Public CI must still be checked after push to verify that GitHub no longer emits
the Node.js 20 runtime warning.

Public CI after push:

```text
run: https://github.com/thromel/ctxhelm/actions/runs/26728393271
head: 8c071cae3561104cce5becaabea1efb9a0767345
workspace job: passed in 1m23s
release-gate smoke job: passed in 1m57s
Node 20 warning grep: no Node.js 20 / node20 / deprecation warning text found
```

## Boundary

This phase does not change the release-gate command set, publish behavior,
artifact contents, or retrieval behavior. It only makes the public CI runner
runtime migration explicit and removes reliance on deprecated Node 20 action
manifests.
