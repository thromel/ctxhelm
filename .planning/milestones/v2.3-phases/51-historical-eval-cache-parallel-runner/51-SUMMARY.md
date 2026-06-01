# Phase 51 Summary

Phase 51 adds source-free historical eval cache reuse, deterministic parallel commit-sample execution, and runtime diagnostics.

## Delivered

- Added `cache_enabled`, `force_refresh`, and `parallelism` to historical eval options and benchmark manifests.
- Added a schema-versioned source-free report cache under local ctxhelm home by repo and eval range.
- Refactored historical eval into deterministic per-commit work units that can execute concurrently and merge in stable sample order.
- Added report/runtime diagnostics for cache hits, cache misses, effective parallelism, git sample cost, ranking cost, pack/compiler cost, and slow commits.
- Updated CLI Markdown output and benchmark suite rendering to expose cache and parallel runner metadata.
- Updated `.ctxhelm/benchmarks/refactoringminer-v23.json` to enable cache reuse and parallelism for the locked v2.3 suite.
- Updated `docs/benchmarking.md` with cache/parallel fields, commands, and runtime interpretation.

## Notes

- Cached reports remain source-free: no source snippets, commit subjects, prompt text, or terminal logs are persisted.
- `--force` recomputes and overwrites the cached report for the same eval range.
- Effective parallelism is capped by the number of commit samples, so a one-sample run reports parallelism `1` even when a higher value is requested.
