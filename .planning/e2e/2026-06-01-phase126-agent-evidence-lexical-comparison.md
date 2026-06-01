# Phase 126: Agent Evidence Lexical Comparison

## Goal

Make the production lexical comparison match the evidence ctxpack actually gives
agents. Phase 125 exposed that target-file-only Recall@10 still trails lexical
on one corpus because tests are intentionally routed through validation outputs.
That is useful honesty, but it is not the adoption-facing evidence set.

## Change

`ctxpack eval proof` now adds agent-evidence fields to
`releaseGate.lexicalComparison` and each corpus verdict.

Agent evidence counts:

- selected context files from `recommendedContextFiles`
- related tests from `recommendedTests`
- test files covered by validation commands

It does not remove or overwrite the older target-file-only `allFileClaim`.

## Proof Result

Command:

```bash
cargo run -p ctxpack -- eval proof --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json --format json
```

Result: `releaseGate.decision = promote`.

Current clean fixture lexical comparison:

- `allFileClaim = trails_any_corpus`
- `agentEvidenceClaim = mixed`
- `contextClaim = mixed`
- all-file corpora: beat `3`, match `0`, trail `1`
- agent-evidence corpora: beat `3`, match `1`, trail `0`
- context-channel corpora: beat `3`, match `1`, trail `0`
- average all-file Recall@10: ctxpack `0.41776595`, lexical `0.45709258`
- average agent-evidence Recall@10: ctxpack `0.64502084`, lexical `0.45709258`
- average context Recall@10: ctxpack `0.64859205`, lexical `0.41836542`

This is the tighter production claim: ctxpack still does not beat lexical on
target-file-only Recall@10, but the actual agent evidence set does not trail
lexical on any corpus and improves average recall by `+0.18792826`.

## Validation

```bash
cargo fmt --all -- --check
bash scripts/check-release-docs.sh
cargo test -p ctxpack-compiler product_proof_release_gate_explains_validation_separated_all_file_divergence -- --nocapture
cargo test -p ctxpack --test cli_compat eval_proof_generates_source_free_product_report -- --nocapture
cargo run -p ctxpack -- eval proof --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json --format json
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p ctxpack -- --help
git diff --check
jq empty .ctxpack/e2e/phase126-agent-evidence-lexical-comparison.json
```

Result: passed.

## Production Notes

- Ranking output is intentionally unchanged.
- The release gate decision remains unchanged.
- The new fields are additive and source-free.
- Future ranking work should still target the remaining target-file-only gap and
  protected target miss rates.
