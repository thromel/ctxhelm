# Phase 174 - Source Recall Release Proof Contract

## Goal

Make Phase 173's source-recall product-proof fields a release artifact freshness contract, not only runtime output. The release checker should reject stale proof JSON that lacks source-channel verdict fields and should block aggregate wins that hide source-code recall regressions.

## Changes

- `scripts/check-product-proof.py` now requires every release-gate corpus verdict to include numeric `sourceRecallAt10`, `lexicalSourceRecallAt10`, and `sourceDeltaAt10`.
- The same checker now fails any corpus with `sourceDeltaAt10 < -0.03`.
- `crates/ctxhelm/tests/release_packaging.rs` covers:
  - promoted proof still accepted;
  - stale proof missing source-recall fields rejected;
  - proof with source-recall regression rejected;
  - existing resource-backed gap and broad fixed-corpus floor checks preserved.
- `docs/benchmarking.md` documents the source-recall fields and release-failure modes.

## Fresh RefactoringMiner Proof

The existing sibling `../RefactoringMiner` checkout was dirty with many deleted files, so Phase 174 used a fresh clone:

- Fresh clone: `/tmp/ctxhelm-rd/RefactoringMiner-fresh`
- Pinned head: `949bddcd3509a805f5e3bcc55fcdb71a691b0dac`
- Pinned base: `c16c031d048618c8389f474f80c2feee21a06ad1`
- Temporary config: `/tmp/ctxhelm-rd/phase174-source-recall-release-contract-config.json`
- Proof artifact: `/tmp/ctxhelm-rd/phase174-source-recall-release-contract-proof.json`

The proof passed `scripts/check-product-proof.py` under the tightened contract and promoted:

| Corpus | File Recall@10 | Status | Source Recall@10 | Lexical Source Recall@10 | Source Delta@10 |
| --- | ---: | --- | ---: | ---: | ---: |
| RefactoringMiner | 0.8 | match | 1.0 | 1.0 | 0.0 |
| ctxhelm | 0.6666667 | beat | 0.46666667 | 0.38333336 | +0.08333331 |
| ReAgent | 0.8 | beat | 1.0 | 0.6666667 | +0.3333333 |
| VeriSchema | 0.18093514 | beat | 0.2763158 | 0.13157895 | +0.14473686 |

## Focused Validation

```bash
cargo test -p ctxhelm --test release_packaging product_proof_checker_accepts_promote_and_rejects_block --locked -- --nocapture
cargo run -p ctxhelm --locked -- eval proof --config /tmp/ctxhelm-rd/phase174-source-recall-release-contract-config.json --format json > /tmp/ctxhelm-rd/phase174-source-recall-release-contract-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-rd/phase174-source-recall-release-contract-proof.json
```

## Full Validation

```bash
cargo fmt --check
bash scripts/check-release-docs.sh
cargo run -p ctxhelm --locked -- --help
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --locked --all-targets -- -D warnings
git diff --check
```
