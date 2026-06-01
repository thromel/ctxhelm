# Phase 125: Lexical Comparison Proof Boundary

## Goal

Make product-proof claims about lexical search explicit and machine-checkable.
The proof gate already separates context and validation channels, but production
release notes still need a compact source-free summary that prevents accidental
"ctxhelm beats lexical everywhere" claims.

## Change

`ctxhelm eval proof` now emits `releaseGate.lexicalComparison` with:

- all-file beat/match/trail counts
- context-channel beat/match/trail counts
- average ctxhelm and lexical all-file Recall@10
- average ctxhelm and lexical context Recall@10
- `allFileClaim`
- `contextClaim`

The Markdown renderer prints the same summary under the release gate decision.

## Proof Result

Command:

```bash
cargo run -p ctxhelm -- eval proof --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json --format json
```

Result: `releaseGate.decision = promote`.

Current clean fixture lexical comparison:

- `allFileClaim = trails_any_corpus`
- `contextClaim = mixed`
- all-file corpora: beat `3`, match `0`, trail `1`
- context-channel corpora: beat `3`, match `1`, trail `0`
- average all-file Recall@10: ctxhelm `0.41776595`, lexical `0.45709258`
- average context Recall@10: ctxhelm `0.64859205`, lexical `0.41836542`

This is the honest production claim: ctxhelm does not currently beat lexical on
macro all-file recall, but it improves the measured context channel while
keeping validation covered.

## Validation

```bash
cargo fmt --all -- --check
bash scripts/check-release-docs.sh
cargo test -p ctxhelm-compiler product_proof_release_gate_blocks_mixed_or_trailing_corpora -- --nocapture
cargo test -p ctxhelm-compiler product_proof_release_gate_explains_validation_separated_all_file_divergence -- --nocapture
cargo test -p ctxhelm --test cli_compat eval_proof_generates_source_free_product_report -- --nocapture
```

Result: passed.

## Production Notes

- Ranking output is intentionally unchanged.
- The release gate decision remains unchanged.
- The new field is additive and source-free.
- Future ranking work can use this summary to show whether a change improves
  all-file behavior, context-channel behavior, or both.
