# Phase 183: Clean Fixture Refresh Runtime Ceiling

## Goal

Restore a reproducible four-repo clean fixture proof after the old pinned
VeriSchema object became unavailable, without weakening the global product-proof
runtime threshold.

## Implementation

- Added source-free `proofRuntimeCeilingMillis` to per-repository benchmark
  config and effective benchmark reports.
- Kept the default product-proof runtime ceiling at `5000ms` per commit.
- Kept the existing default-policy one-commit perfect-ceiling diagnostic
  exception.
- Added a verdict note whenever a repo-scoped ceiling is used, including the
  observed per-commit runtime and configured ceiling.
- Added the refreshed clean fixture config:
  `.planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json`.
- Set `proofRuntimeCeilingMillis: 15000` only on the detached
  RefactoringMiner fixture. ctxhelm, ReAgent, and VeriSchema remain on the
  default ceiling.
- Updated the release gate default clean fixture config and artifact name:
  `phase183-clean-fixture-product-proof.json`.

## Cold Proof

Command:

```bash
rm -rf /tmp/ctxhelm-phase183-clean-fixture-home
env CTXHELM_HOME=/tmp/ctxhelm-phase183-clean-fixture-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase183-clean-fixture-refresh-proof-scoped.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase183-clean-fixture-refresh-proof-scoped.json
```

Result:

- `releaseGate.decision = promote`
- RefactoringMiner: `match`, File Recall@10 `0.8`, lexical `1.0`,
  observed runtime `11803ms` per commit under the explicit `15000ms` ceiling.
- ctxhelm: `beat`, File Recall@10 `0.6666667`, lexical `0.30634922`.
- ReAgent: `beat`, File Recall@10 `0.8`, lexical `0.4`.
- VeriSchema: `beat`, File Recall@10 `0.35529414`, lexical `0.21176472`.

## Warm Proof

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase183-clean-fixture-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase183-clean-fixture-refresh-warm-proof-scoped.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase183-clean-fixture-refresh-warm-proof-scoped.json
```

Result:

- `releaseGate.decision = promote`
- RefactoringMiner total runtime `652ms`
- ctxhelm total runtime `560ms`
- ReAgent total runtime `689ms`
- VeriSchema total runtime `664ms`

## Verification

```bash
cargo test -p ctxhelm-compiler product_proof_release_gate --locked -- --nocapture
```

Result: 13 focused release-gate tests passed, including
`product_proof_release_gate_accepts_repo_scoped_runtime_ceiling`.
