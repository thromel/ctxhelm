---
phase: 65
title: v2.5 Product Proof And Release Gate
status: completed
completed_at: 2026-05-30T07:05:56Z
requirements_addressed:
  - PROOF-01
  - PROOF-02
  - PROOF-03
  - PROOF-04
---

# Phase 65 Summary

## Completed

- Added `releaseGate` to product proof JSON and Markdown.
- Added source-free per-corpus verdicts for the configured retrieval variant:
  `beat`, `match`, `trail`, or `insufficient_evidence`.
- Added release decision logic that blocks non-promote proof results.
- Extracted product-proof validation to `scripts/check-product-proof.py`.
- Wired optional `CTXHELM_BENCHMARK_CONFIG` release-gate proof to fail when the
  proof does not promote.
- Updated benchmarking and release docs with current v2.5 recommendation.

## Measured Outcome

The current v2.5 two-repo proof blocks default promotion:

- RefactoringMiner: `trail`, ctxhelm Recall@10 `0.7392`, lexical Recall@10
  `0.7792`, delta `-0.0400`.
- ctxhelm: `match`, ctxhelm Recall@10 `0.1974`, lexical Recall@10 `0.2021`,
  delta `-0.0046`.
- Decision: `block`.
- Default promotion allowed: `false`.

## Validation

```bash
cargo test -p ctxhelm-compiler product_proof_release_gate -- --nocapture
cargo test -p ctxhelm --test release_packaging -- --nocapture
cargo run -p ctxhelm -- eval proof --config .ctxhelm/e2e/phase62-default-config.json --format json
python3 scripts/check-product-proof.py .ctxhelm/e2e/phase65-product-proof.json
bash scripts/check-release-docs.sh
cargo test --workspace --no-fail-fast
cargo run -p ctxhelm -- --help
git diff --check
```

The proof checker rejects the current v2.5 proof as expected because
`releaseGate.decision` is `block`.

## Release Position

v2.5 is production-ready as a measured, local-first context broker with honest
release proof. It is not production-ready as a lexical-beating retrieval default.
The gate now prevents that claim from shipping prematurely.
