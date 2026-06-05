# Phase 249 Context Governor Proof

Date: 2026-06-06

## Scope

Phase 249 adds `ctxhelm governor decide`, a source-free context-governor report
that explains task-conditioned retrieval, budget, memory, validation, semantic,
and policy-profile decisions.

The report contract is `ctxhelm-context-governor-report-v1`.

## Evidence

- `crates/ctxhelm-core/src/contracts.rs` adds `ContextGovernorReport`,
  `ContextGovernorDecision`, `ContextGovernorEvidence`, and
  `ContextGovernorRolloutControl`.
- `crates/ctxhelm/src/main.rs` adds `governor decide`, report building, and
  Markdown/JSON rendering.
- `docs/context-governor.md` documents the command, report fields, rollout
  flow, and source-free privacy boundary.
- `scripts/smoke-governor.sh` creates a temporary repo, records source-free
  feedback, runs `ctxhelm governor decide`, tunes/applies/rolls back a policy
  profile, reruns the governor, and rejects source sentinel leakage.
- `scripts/release-gate.sh` now runs `scripts/smoke-governor.sh` and records it
  in required release checks.

## Requirement Mapping

- GOVERN-01: adaptive retrieval/budget/memory/validation policy per task is
  visible through governor decision areas.
- GOVERN-02: source-free feedback and policy profile counts are included, and
  the smoke uses feedback-derived policy tuning.
- GOVERN-03: rollout controls include learn, compare, apply, and rollback; the
  smoke proves apply/rollback visibility in later governor reports.
- GOVERN-04: selected and omitted evidence are machine-readable in the report.

## Validation

Passed:

```bash
bash scripts/smoke-governor.sh
bash scripts/check-release-docs.sh
cargo test -p ctxhelm --test release_packaging --locked
cargo fmt --check
cargo check -p ctxhelm --locked
```

Additional full-workspace validation is still required before claiming a final
release-ready state.
