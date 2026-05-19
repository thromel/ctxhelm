# Phase 54 Context: Offline Learned Retrieval Policy Experiment

## Goal

Maintainers can generate, compare, apply, disable, and roll back non-default
learned retrieval-policy profiles from source-free local evidence.

## Inputs

- Phase 52 candidate feature export rows.
- Phase 53 paired baseline and ablation reports.
- Local feedback events and outcome comparison reports.
- Existing policy profile lifecycle: tune, list, apply, disable, rollback.

## Constraints

- Learned profiles must stay local and source-free.
- Profiles must remain candidates until explicitly applied.
- Profiles must not become active unless configured baseline thresholds pass.
- Safety floors must preserve anchors, lexical identifiers, and validation
  context.

## Implementation Areas

- `crates/ctxpack-core/src/contracts.rs`
- `crates/ctxpack-index/src/feedback.rs`
- `crates/ctxpack-index/src/lib.rs`
- `crates/ctxpack/src/main.rs`
- `crates/ctxpack/tests/cli_compat.rs`
- `docs/feedback.md`
- `docs/feature-exports.md`
- `docs/learned-policy.md`

