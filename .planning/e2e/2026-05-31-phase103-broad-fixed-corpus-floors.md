# Phase 103: Broad Fixed-Corpus Floors

## Goal

Prevent broad four-repo proof regressions from being accepted only because the
aggregate product-proof gate still promotes.

## Problem

A Phase 103 ranking experiment moved broad-scope dependency floors ahead of
symbol overflow candidates. Focused dependency tests passed, but the pinned
four-repo proof regressed VeriSchema:

- File Recall@10: `0.18449473` -> `0.17936651`
- Source Recall@10: `0.31067252` -> `0.30409357`
- Test Recall@10 and Effective Validation Recall@10 stayed flat

The existing product-proof checker still accepted the report because the
release gate promoted overall. That made broad fixed-corpus proof artifacts too
weak as regression guards for selection and budget pressure.

## Implementation

- Rejected the dependency-priority ranking change and left retrieval behavior
  unchanged.
- Added pinned broad fixed-corpus floors to `scripts/check-product-proof.py`
  for corpus ID `phase92-area-aware-gap-taxonomy-2026-05-31`.
- The checker now validates per-repository floors for the currently proven
  RefactoringMiner, ctxpack, ReAgent, and VeriSchema file/source/test,
  effective-validation, and broad-context-area metrics.
- Added release-packaging tests proving an at-floor proof passes and a
  VeriSchema `fileRecallAt10 = 0.17936651` regression fails with a broad
  fixed-corpus floor error.

## Validation

```bash
cargo test -p ctxpack --test release_packaging product_proof_checker_accepts_promote_and_rejects_block -- --nocapture
python3 scripts/check-product-proof.py .ctxpack/e2e/phase100-resource-backed-gap-summaries-proof.json
python3 scripts/check-product-proof.py .ctxpack/e2e/phase103-broad-dependency-priority-proof.json
```

The Phase 100 proof passes. The rejected Phase 103 ranking proof fails as
expected with:

```text
broad fixed corpus metric regressed below floor: VeriSchema.fileRecallAt10 0.17936651 < 0.18449473
```

## Result

The broad proof now has two layers:

- `releaseGate.decision = promote` still proves the overall product-proof
  verdict.
- Pinned broad fixed-corpus floors prevent accepted reports from silently
  regressing known-good repository metrics on the current four-repo corpus.

Next ranking work should address ranked-below-budget source/docs pressure with
proofs that improve or preserve these floors, not just pass the aggregate gate.
