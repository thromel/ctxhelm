# Phase 303 - Retention Separator Margin Diagnostic

## Scope

Phase 302 found zero eligible strict retention-separator train families across
four recent-to-older proofs. Before widening or relaxing that separator, Phase
303 adds the missing margin diagnostic recommended by Claude Code: measure how
far the source-free retained-target / retained-non-target family distribution
misses the strict gate.

This phase extends `ctxhelm eval retention-separator-train-test` with additive
schema-version-2 fields:

- `positiveMarginFamilyCount`
- `positiveMarginFamilyRatio`
- `supportDeficientCleanFamilyCount`
- `churnBlockedSupportedFamilyCount`
- `thresholdSummaries`
- per-profile `retentionMargin`

The strict eligibility rule and report decision are unchanged. The threshold
sweep is diagnostic-only and scans retained target commit support versus
retained non-target tolerance to answer whether the `0 eligible` result is a
miscalibrated gate or an inseparable proxy.

## Proof Command

After a feature-enabled build:

```bash
cargo build -p ctxhelm --features local-embeddings --locked
```

Each report used the same stable Phase 302 train/test ranges:

```bash
./target/debug/ctxhelm eval retention-separator-train-test \
  --repo <fixture> \
  --train-base <recent-base> --train-head <recent-head> \
  --test-base <older-base> --test-head <older-head> \
  --train-limit 20 --test-limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-model JinaEmbeddingsV2BaseCode \
  --semantic-query-mode candidate-path-hints \
  --format json
```

Artifacts were written under `.ctxhelm/e2e/phase303-retention-separator-margin-*`
and remain ignored proof data.

## Result

Across ctxhelm, ReAgent, RefactoringMiner, and VeriSchema:

| Metric | Value |
| --- | ---: |
| train families | `117` |
| eligible strict families | `0` |
| positive-margin families | `8` |
| positive-margin ratio | `0.068376` |
| support-deficient clean families | `5` |
| churn-blocked supported families | `3` |
| test dropped profiles | `377` |
| train/test family overlap | `53` |
| strict applied files | `0` |
| strict recovered targets | `0` |
| strict inserted non-targets | `0` |

Per repo:

| Repo | Train families | Positive-margin families | Ratio | Support-deficient clean | Churn-blocked supported |
| --- | ---: | ---: | ---: | ---: | ---: |
| ctxhelm | 36 | 7 | `0.194444` | 5 | 2 |
| ReAgent | 31 | 0 | `0.000000` | 0 | 0 |
| RefactoringMiner | 14 | 0 | `0.000000` | 0 | 0 |
| VeriSchema | 36 | 1 | `0.027778` | 0 | 1 |

The aggregate positive-margin ratio is below the pre-registered `15%`
threshold for continuing separator-feature work. Relaxed threshold summaries
also confirm the structural failure: the best aggregate recovery row
(`minRetainedTargetCommitCount = 1`, `maxRetainedNonTargetCount = 3`) recovers
only `3` dropped targets while inserting `21` non-targets. The cleanest relaxed
row that recovers any target (`1`, `0`) still recovers `1` target while
inserting `4` non-targets.

## Decision

Keep semantic retention separation diagnostic-only. The margin distribution
shows that Phase 302's zero eligible families are not a simple threshold
calibration issue. Do not widen or relax retention separators as the next
semantic branch. The next high-value R&D path should pivot to agent outcome
proof, especially fresh Claude Code held-out outcome evidence when the client is
available, or a materially different source-free feature family with a new
pre-registered acceptance bar.
