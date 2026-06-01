# Phase 60 Summary: Semantic/Precision Evaluation Gates And Release Proof

## Outcome

Phase 60 is complete.

ctxhelm now has a source-free semantic/precision release gate that reports
whether semantic, precision, and reranker variants should be promoted, held as
opt-in, or blocked.

## Implemented

- Added `ctxhelm eval gate`.
- Added `SemanticPrecisionGateReport` with:
  - promote/hold/block decision
  - variant rows
  - provider policy
  - precision status
  - Recall@K, precision proxy, MRR, Test Recall@10, runtime/cache fields, token
    efficiency
  - named wins, named regressions, and named misses
- Added deterministic variants:
  - `lexical_baseline`
  - `ctxhelm_default`
  - `local_semantic`
  - `precision_enriched_semantic`
  - `semantic_precision_full_hybrid`
  - `policy_allowed_reranked`
- Policy-blocked variants are reported as `skipped`.
- Unsafe provider policy, named regressions, or metric regressions block.
- Neutral/mixed outcomes hold features as opt-in.
- Added `scripts/smoke-v24-gate.sh`.
- Added the v2.4 gate smoke to `scripts/release-gate.sh`.
- Updated release docs, benchmarking docs, semantic docs, and precision docs.

## Verification

- `cargo test -p ctxhelm-compiler eval --no-fail-fast`
- `cargo test -p ctxhelm-compiler gate --no-fail-fast`
- `CTXHELM_BIN=target/debug/ctxhelm bash scripts/smoke-v24-gate.sh`
- `CTXHELM_BIN=target/debug/ctxhelm bash scripts/smoke-v23-eval.sh`
- `bash scripts/check-release-docs.sh`
- `cargo test --workspace --no-fail-fast`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`

## Notes

- A `hold` or `block` gate decision is a valid release outcome when docs plainly
  state that semantic/precision/reranker behavior remains opt-in.
- The gate is conservative by design. It treats measured evidence, named cases,
  provider policy, and privacy status as release inputs.
- v2.4 does not claim semantic/precision default promotion; it now has the gate
  needed to make or reject that decision with evidence.
