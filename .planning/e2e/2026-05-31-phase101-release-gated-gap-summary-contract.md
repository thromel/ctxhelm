# Phase 101: Release-Gated Gap Summary Contract

## Goal

Make the Phase 100 resource-backed retrieval-gap shape part of the product-proof
release contract, not just an optional report field.

## Why

Phase 100 added source-free `contextAreaResourceUri` and `nextReadPaths` fields
to current reachable retrieval-gap summaries. Without a release-gate assertion,
future changes could silently keep `releaseGate.decision = promote` while
dropping the progressive-read evidence that agents need to act on measured
misses.

## Implementation

- Extended `scripts/check-product-proof.py` to inspect embedded benchmark
  `report.retrievalGapSummaries`.
- For each gap with `targetStatus = currentReachable`, the checker now requires:
  - `contextAreaResourceUri` beginning with `ctxpack://repo/context-area/`
  - non-empty `nextReadPaths`
  - non-blank string entries in `nextReadPaths`
- Added a release-packaging regression test that proves promoted product proofs
  pass with resource-backed gaps and fail when a current reachable gap drops the
  context-area resource URI.

## Validation

```bash
cargo test -p ctxpack product_proof_checker_accepts_promote_and_rejects_block -- --nocapture
python3 scripts/check-product-proof.py .ctxpack/e2e/phase100-resource-backed-gap-summaries-proof.json
```

## Result

- The existing Phase 100 four-repo proof passes the stricter checker.
- Current reachable gap summaries are now release-gated for agent-consumable
  progressive-read evidence.
- No ranking, privacy, or benchmark metric behavior was changed.
