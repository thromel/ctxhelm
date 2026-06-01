# Phase 111 - Clean Fixture Release Gate

Date: 2026-06-01

## Goal

Make the Phase 110 clean cold four-repo proof part of the packaged release-gate path instead of a manual follow-up artifact.

## Change

- `scripts/release-gate.sh` now has a named clean cold fixture proof step.
- The default config is `.planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json`.
- If detached fixtures exist, the gate writes `phase110-clean-fixture-product-proof.json` into `CTXHELM_PROOF_DIR` and validates it with `scripts/check-product-proof.py`.
- `CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1` makes missing fixtures fail the gate.
- `CTXHELM_CLEAN_FIXTURE_CONFIG` can override the config path.
- `CTXHELM_SKIP_CLEAN_FIXTURE_PROOF=1` explicitly skips the check for non-release local diagnostics.
- The release proof summary records `cleanColdFixtureProductProof` and `cleanColdFixtureRequired`.

## Fixture Preparation

Run:

```bash
bash scripts/prepare-proof-fixtures.sh
```

This prepares clean detached checkouts under `../ctxhelm-proof-fixtures`.

## Proof

Command:

```bash
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxhelm-phase111-target \
  cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json \
  --format json >/tmp/ctxhelm-phase111-clean-fixture-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase111-clean-fixture-proof.json
```

Result:

- `releaseGate.decision = promote`
- RefactoringMiner: `match`, context Recall@10 `1.0`, lexical context Recall@10 `1.0`
- ctxhelm: `beat`, context Recall@10 `0.3888889`, lexical context Recall@10 `0.30555555`
- ReAgent: `beat`, context Recall@10 `1.0`, lexical context Recall@10 `0.2857143`
- VeriSchema: `beat`, context Recall@10 `0.20547946`, lexical context Recall@10 `0.08219178`

## Focused Validation

- `bash -n scripts/release-gate.sh`
- `bash -n scripts/check-release-docs.sh`
- `bash scripts/check-release-docs.sh`
- `cargo fmt --check`
- `CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxhelm-phase111-target cargo test -p ctxhelm --test release_packaging -- --nocapture`

## Notes

The local checkout had macOS `dataless` files under docs and crate manifests. Small manifest files were hydrated for Cargo validation. `scripts/check-release-docs.sh` now avoids Git lookups for dataless files so the release-doc check does not hang on local cloud-file placeholders.
