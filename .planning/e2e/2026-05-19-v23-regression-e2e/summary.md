# v2.3 Regression E2E Summary

## Corpus

- Suite: `v2.3-refactoringminer-regression-clean-e2e`
- Repository: `RefactoringMiner-clean`
- Evaluated commits: `20`
- Corpus ID: `ctxhelm-v2.3-refactoringminer-20-commit-clean-e2e`
- Privacy: local-only `True`

## Current Metrics

- `fileRecallAt10`: `0.5186`
- `lexicalBaselineRecallAt10`: `0.5008`
- `ctxhelmLiftAt10`: `0.0179`
- `testRecallAt10`: `0.4722`
- `runtime.totalMillis`: `61832`
- `runtime.averageCommitMillis`: `10072.50`
- `runtime.parallelism`: `4`
- `runtime.cacheHits`: `0`
- `runtime.cacheMisses`: `1`

## Baseline Deltas

- Locked baseline Recall@10: `0.5186`
- Current Recall@10 delta: `+0.0000`
- Locked lexical Recall@10: `0.5008`
- Current lexical Recall@10 delta: `-0.0000`
- Locked runtime total ms: `265650`
- Current runtime delta ms: `-203818`

## Product Proof

- `averageFileRecallAt10`: `0.5186` `ratio`
- `averageLexicalBaselineRecallAt10`: `0.5008` `ratio`
- `averageCtxpackLiftAt10`: `0.0179` `ratio`
- `averageTestRecallAt10`: `0.4722` `ratio`
- `averageBriefTokenRoi`: `5.7500` `useful_targets_per_1k_tokens`
- Fixed corpus ID: `ctxhelm-v2.3-refactoringminer-20-commit-clean-e2e`
- Paired baseline verdicts: `1`
- Lexical status: `neutral`, delta@K `+0.0179`
- Feature export local-only: `True`
- Learned policy default requires thresholds: `True`

## Interpretation

- Neutral: current ctxhelm Recall@10 is within 1 percentage point of the locked baseline.
- Runtime is materially faster than the locked baseline.

## Regression Sweep

- `CARGO_INCREMENTAL=0 cargo test --workspace`: passed.
- `bash scripts/check-release-docs.sh`: passed.
- `CTXHELM_BIN=target/debug/ctxhelm bash scripts/smoke-v23-eval.sh`: passed.
- `CTXHELM_BIN=target/debug/ctxhelm CTXHELM_SMOKE_REPO=$PWD bash scripts/smoke-historical-eval.sh`: passed for ctxhelm's own 3-commit smoke corpus.
- RefactoringMiner 20-commit clean-worktree benchmark: passed with no quality regression versus locked baseline.
- `CARGO_TARGET_DIR=/tmp/ctxhelm-release-gate-target CARGO_INCREMENTAL=0 CTXHELM_ALLOW_DIRTY=1 CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/release-gate.sh`: passed.

## Findings

- Fixed product regression: explicit path anchors could be crowded out of `targetFiles` when a weak task prompt produced many strong lexical matches. The anchor still appeared in `retrievalCandidates`, but the lexical floor consumed the target-file budget first. `select_target_files` now pins explicit/current-diff anchors before lexical and history floors.
- Added regression coverage: `ranking::tests::selection_preserves_explicit_anchors_before_lexical_floor`.
- Verified the exact failing Claude wrapper protocol case now passes: `CTXHELM_SMOKE_TASK="e2e claude wrapper smoke"` with `CTXHELM_SMOKE_PATH="crates/ctxhelm/src/main.rs"`.
- Environment note: default Cargo incremental build hit a transient `target/debug/incremental/.../dep-graph.part.bin` creation error. Rerunning with `CARGO_INCREMENTAL=0` passed.
- Environment note: release packaging on the existing `target/release` cache wedged in macOS `fclonefileat`. A fresh `CARGO_TARGET_DIR` completed the full release gate.
