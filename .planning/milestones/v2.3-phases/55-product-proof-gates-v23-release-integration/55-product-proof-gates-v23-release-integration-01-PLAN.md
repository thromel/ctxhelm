# Plan: Product Proof Gates & v2.3 Release Integration

## Objective

Wire the v2.3 evaluation lab into product proof, release docs, and release
gates so a maintainer can run deterministic source-free proof locally while
keeping large external repos optional.

## Tasks

1. Extend product proof JSON and Markdown with fixed corpus identity, paired
   baseline verdicts, runtime diagnostics, feature-export privacy, learned
   policy status, and proof-boundary language.
2. Add a deterministic `scripts/smoke-v23-eval.sh` that creates a small local
   git corpus and exercises feature export, feedback recording, learned policy,
   paired baselines, and product proof.
3. Add v2.3 smoke execution and JSON contract checks to `scripts/release-gate.sh`.
4. Update release-packaging and CLI contract tests for the new proof fields and
   release-gate check.
5. Update benchmarking and release docs, including optional RefactoringMiner
   and multi-repo commands plus clear skip semantics.
6. Update GSD requirement, roadmap, summary, and verification artifacts.

## Verification

- `cargo fmt --all`
- `CARGO_INCREMENTAL=0 cargo check --workspace`
- `CARGO_INCREMENTAL=0 cargo test -p ctxhelm --test cli_compat eval_proof_generates_source_free_product_report`
- `CARGO_INCREMENTAL=0 cargo test -p ctxhelm --test release_packaging release_gate_script_contract`
- `CARGO_INCREMENTAL=0 cargo run -p ctxhelm -- eval proof --help`
- `CARGO_INCREMENTAL=0 cargo run -p ctxhelm -- --help`
- `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" bash scripts/smoke-v23-eval.sh`
- `CARGO_INCREMENTAL=0 cargo test --workspace`
- `git diff --check`
- `gsd-sdk query roadmap.analyze`
