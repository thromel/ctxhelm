# Phase 55 Context: Product Proof Gates & v2.3 Release Integration

## Goal

Maintainers can include bounded v2.3 evaluation proof in docs and release gates
without requiring external repositories by default.

## Inputs

- Phase 50 fixed benchmark corpus manifests and optional RefactoringMiner suite.
- Phase 51 cached historical eval runtime diagnostics.
- Phase 52 source-free candidate feature export.
- Phase 53 paired baseline and ablation verdicts.
- Phase 54 learned retrieval-policy profile status and threshold controls.
- Existing release gate, release docs, and product proof CLI.

## Constraints

- The mandatory release path must not require RefactoringMiner or any external
  multi-repo checkout.
- Product proof must remain local-only and source-free.
- RefactoringMiner and larger corpus proof remain optional external gates with
  explicit commands and skip reasons.
- The docs must distinguish useful retrieval at lexical parity from
  world-class measured lift.

## Implementation Areas

- `crates/ctxpack-compiler/src/eval.rs`
- `crates/ctxpack/src/main.rs`
- `crates/ctxpack/tests/cli_compat.rs`
- `crates/ctxpack/tests/release_packaging.rs`
- `scripts/smoke-v23-eval.sh`
- `scripts/release-gate.sh`
- `scripts/check-release-docs.sh`
- `docs/benchmarking.md`
- `docs/release.md`
