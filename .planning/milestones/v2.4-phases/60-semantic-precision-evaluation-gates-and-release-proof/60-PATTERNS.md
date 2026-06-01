---
phase: 60
title: Semantic Precision Evaluation Gates And Release Proof
date: 2026-05-20
status: patterns
---

# Phase 60 Patterns

## Existing Patterns To Reuse

- Keep eval reports deterministic and JSON-first.
- Extend existing benchmark/product proof contracts instead of creating a separate release format.
- Keep smoke scripts local and fast by default, with optional real-repo paths.
- Product proof should make claims only from measured reports.

## Closest Code Analogs

- `crates/ctxhelm-compiler/src/eval.rs`: historical eval, paired baselines, benchmark suite, product proof.
- `scripts/smoke-v23-eval.sh`: fixed-corpus eval proof.
- `docs/benchmarking.md`: current eval docs and report interpretation.
- `docs/release.md`: release proof documentation.

## Implementation Notes

- Add variant controls after Phases 57-59 are implemented.
- Treat policy-blocked reranker/cloud variants as expected skipped variants, not failures.
- Require named regression output so mixed results are actionable.
- Do not update release claims unless the gate output supports them.
