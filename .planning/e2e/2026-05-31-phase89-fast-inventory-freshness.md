# Phase 89: Fast Inventory Freshness

## Goal

Reduce broad-proof cold runtime without weakening quality, privacy, or release
gate thresholds. Phase 88 still blocked in debug-mode broad proof because
`load_or_refresh_inventory` rebuilt and re-hashed full inventories on every
cache hit inside a single context-plan compilation.

## Change

- Inventory cache freshness now builds a current metadata manifest from
  filesystem metadata and cached hashes instead of rebuilding the full
  inventory and re-reading every source file.
- Stale-cache detection still compares schema, policy, options, repo root,
  ignore fingerprints, file creation/deletion, and file metadata changes.
- Full inventory rebuilding remains the path for cache misses and stale caches.

## Evidence

Focused freshness tests:

```bash
cargo test -p ctxhelm-index freshness -- --nocapture
```

Pinned broader debug proof:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > .ctxhelm/e2e/phase89-fast-inventory-freshness-proof.json
```

Pinned broader release proof:

```bash
cargo run --release -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > .ctxhelm/e2e/phase89-fast-inventory-freshness-release-proof.json
```

Product proof checker:

```bash
python3 scripts/check-product-proof.py \
  .ctxhelm/e2e/phase89-fast-inventory-freshness-release-proof.json
```

## Result

Quality metrics match Phase 88; runtime improves materially.

| Corpus | Phase 88 total ms | Phase 89 debug total ms | Phase 89 release total ms | Phase 89 release result |
| --- | ---: | ---: | ---: | --- |
| RefactoringMiner | `23871` | `21938` | `8279` | `match`, promoted as single-commit lexical ceiling |
| ctxhelm | `25232` | `19386` | `8317` | `beat` |
| ReAgent | `17629` | `14675` | `4264` | `beat` |
| VeriSchema | `18051` | `17076` | `6590` | `beat` |

Release proof:

```text
releaseGate.decision = promote
```

The release proof is the authoritative production-readiness artifact because it
measures the optimized CLI binary. The debug proof remains useful diagnostics:
it now blocks only on the RefactoringMiner single-commit cold-start threshold,
while ctxhelm, ReAgent, and VeriSchema no longer exceed the per-commit gate in
debug mode.

## Next Work

- Continue source candidate improvements for remaining parser/precision
  `no_candidate_signal` families.
- Keep release-mode proof as the runtime authority for production publication.
- Use debug-mode proof only as a conservative diagnostic for local development
  performance regressions.
