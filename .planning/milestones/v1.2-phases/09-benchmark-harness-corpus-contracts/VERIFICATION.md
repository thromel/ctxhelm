---
phase: 9
status: passed
verified: 2026-05-14
---

# Verification: Phase 9 Benchmark Harness & Corpus Contracts

## Verdict

Passed.

Phase 9 established named benchmark-suite contracts, multi-repo benchmark execution, reproducibility metadata, source-free privacy boundaries, CLI access, and setup documentation.

## Requirements

| Requirement | Status | Evidence |
|-------------|--------|----------|
| BENCH-01 | Passed | `BenchmarkSuiteConfig`, `BenchmarkRepoConfig`, `BenchmarkDefaults`, role filters, revision range, limit, and ranking budget fields |
| BENCH-02 | Passed | `run_benchmark_suite` executes bounded historical evals across multiple configured repos |
| BENCH-03 | Passed | `BenchmarkSuiteReport` includes repo IDs, commit counts, effective config, skipped/excluded path counts, and privacy status without source text |
| BENCH-04 | Passed | `docs/benchmarking.md` documents adding local benchmark repos, including RefactoringMiner-style setup |

## Commands

```bash
cargo check --workspace
cargo test -p ctxpack-compiler benchmark_suite_runs_multiple_repos_with_source_free_metadata -- --nocapture
cargo test -p ctxpack --test cli_compat eval_benchmark_runs_named_suite_source_free -- --nocapture
cargo run -p ctxpack -- eval benchmark --help
```

## Notes

Role filters are recorded in the suite contract and effective report metadata in Phase 9. Deeper role-filtered segmentation belongs to Phase 10 and Phase 11.
