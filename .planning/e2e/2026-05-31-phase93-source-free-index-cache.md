# Phase 93: Source-Free Index Cache

## Goal

Reduce cold large-repo planner runtime without changing release thresholds,
shipping source-bearing cache artifacts, or weakening the broad fixed-corpus
quality gates.

Phase 92 proved the clean four-repo fixture in warm-cache mode, but the
force-refresh proof still documented that clean RefactoringMiner exceeded the
hard cold runtime ceiling without cached historical reports. Phase 93 targets
that bottleneck directly by caching source-free derived indexes that the planner
rebuilds repeatedly during historical eval:

- symbol extraction reports
- dependency edge reports

Both caches are keyed by inventory file path, hash, role, language, and the
relevant cache-version marker. Cache files live under the repo-local ctxhelm
cache area derived from the inventory path, not inside the source tree.

## Implementation

- Added a source-free symbol extraction cache in
  `crates/ctxhelm-index/src/symbols.rs`.
- Added a source-free dependency edge cache in
  `crates/ctxhelm-index/src/dependencies.rs`.
- Added focused cache invalidation tests in
  `crates/ctxhelm-index/src/lib.rs`.

The caches return `CacheStatusKind::Hit` on reuse and invalidate when the
inventory changes. Persist failures are non-fatal, matching the existing
source-free cache behavior.

## Evidence

Focused tests:

```bash
cargo test -p ctxhelm-index symbol_search_reuses_extraction_cache_until_inventory_changes -- --nocapture
cargo test -p ctxhelm-index dependency_edges_reuse_cache_until_inventory_changes -- --nocapture
```

Broad cold-runtime proof:

```bash
cargo run --release -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > /tmp/ctxhelm-phase93-index-cache-force-populate.json

cargo run --release -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > /tmp/ctxhelm-phase93-index-cache-force-proof.json

python3 scripts/check-product-proof.py \
  .ctxhelm/e2e/phase93-index-cache-cold-proof.json
```

Committed proof:

- `.ctxhelm/e2e/phase93-index-cache-cold-proof.json`

Result:

- `releaseGate.decision = promote`
- RefactoringMiner runtime: `4517ms`
- RefactoringMiner status: `match`
- RefactoringMiner context Recall@10: `1.0`
- RefactoringMiner Test Recall@10: `1.0`
- RefactoringMiner Effective Validation Recall@10: `1.0`
- RefactoringMiner protected target miss-rate@10: `0.0`
- VeriSchema broad context-area recall: `0.64708996`
- VeriSchema Effective Validation Recall@10: `1.0`

The key production signal is that the same clean detached RefactoringMiner
fixture that exceeded the hard cold runtime ceiling in Phase 92 now promotes
without threshold changes.

## Notes

The broader `ctxhelm` and `ReAgent` rows still report higher total runtimes, but
the Phase 92 blocker was specifically the clean RefactoringMiner cold diagnostic
under force-refresh proof. This phase removes that blocker while preserving the
same broader quality metrics and local-only proof contract.
