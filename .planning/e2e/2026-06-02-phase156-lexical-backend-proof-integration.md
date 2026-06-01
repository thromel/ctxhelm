# Phase 156 - Lexical Backend Product Proof Integration

## Goal

Make the Phase 155 BM25-vs-legacy corpus comparison visible in benchmark and
product-proof reports without making the comparison part of the default proof
cost.

## Implementation

- Added optional `lexicalBackendComparison` to benchmark defaults and per-repo
  config.
- Added `lexicalBackendCorpus` and source-free `lexicalBackendError` fields to
  benchmark repository reports.
- Added `releaseGate.lexicalBackendComparison` to product proof, aggregating
  successful backend corpus reports across repositories.
- Rendered the repository-level backend comparison in benchmark Markdown and the
  aggregate backend summary in product-proof Markdown.
- Documented the opt-in flag in benchmark, paired-baseline, and release docs.

## Debugging Notes

The first focused proof test failed because the fixture asserted
`lexicalBackendCorpus` but did not enable `lexicalBackendComparison` in the
proof suite defaults. The flag had been added to a neighboring eval-compare
fixture instead. The corrected proof fixture now opts in explicitly and verifies
both the embedded repository comparison and the release-gate aggregate.

## Source-Free Contract

The new proof path keeps task text and source text out of report output:

- backend rows use task hashes rather than commit subjects;
- repository reports store paths, hits, timing, and source-free metrics;
- backend setup failures are recorded as `lexicalBackendError`;
- product proof aggregates only metrics and thresholded claims.

## Validation

- `cargo test -p ctxhelm --test cli_compat eval_proof_generates_source_free_product_report --locked` passed.
- `cargo test -p ctxhelm --test cli_compat eval_lexical --locked` passed.
- `cargo check -p ctxhelm --locked` passed.
- `cargo fmt --check` passed.
- `bash scripts/check-release-docs.sh` passed.
- `cargo run -p ctxhelm --locked -- --help` passed.
- `git diff --check` passed.
- `cargo test --workspace --locked --no-fail-fast` passed.
- `cargo clippy --workspace --locked --all-targets -- -D warnings` passed.
