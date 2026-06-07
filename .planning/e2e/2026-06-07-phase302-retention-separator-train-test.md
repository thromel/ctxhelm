# Phase 302 - Retention Separator Train/Test

## Scope

Phase 301 made supported semantic candidate retention observable, but the
aggregate still had retained non-target pressure. Phase 302 tests the next
pre-registered path: train a source-free retention-family separator on one
revision range and apply it to dropped supported semantic candidates in a
disjoint test range.

This phase adds `ctxhelm eval retention-separator-train-test` and the
`SemanticRetentionSeparatorTrainTestReport` schema. The report is diagnostic
only and does not change ranking or runtime policy.

Training keys use the same retention family dimensions as Phase 301:

- query family
- file role
- path family
- support family

A family is eligible only when it has retained target support in at least two
training commits and zero retained non-targets. Test application considers only
dropped supported semantic candidates from the disjoint test range. The report
counts applied files, recovered dropped targets, inserted non-targets, family
overlap, and profile decision reasons.

## Proof Command

The proof artifacts were regenerated after a clean feature-enabled build:

```bash
cargo build -p ctxhelm --features local-embeddings --locked
```

Each report used:

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

## Result

Across ctxhelm, ReAgent, RefactoringMiner, and VeriSchema recent-to-older
train/test reports:

| Metric | Value |
| --- | ---: |
| reports | `4` |
| train families | `117` |
| eligible families | `0` |
| test dropped profiles | `377` |
| train/test family overlap | `53` |
| train/test eligible family overlap | `0` |
| applied commits | `0` |
| applied files | `0` |
| recovered dropped targets | `0` |
| inserted non-targets | `0` |

Per-repo decisions:

| Repo | Train families | Eligible families | Test dropped profiles | Family overlap | Applied files | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| ctxhelm | 36 | 0 | 142 | 22 | 0 | `insufficient_train_families` |
| ReAgent | 31 | 0 | 121 | 12 | 0 | `insufficient_train_families` |
| RefactoringMiner | 14 | 0 | 34 | 7 | 0 | `insufficient_train_families` |
| VeriSchema | 36 | 0 | 80 | 12 | 0 | `insufficient_train_families` |

The diagnostic is still useful because it explains why the strict separator
does not apply. The overlapping dropped-target rows are mostly blocked by
`no_retained_training_targets` or `insufficient_retained_target_commit_support`;
several rows also show retained training non-target pressure. That means the
Phase 301 retention signal is not yet durable enough for a no-churn separator.

## Decision

Keep semantic retention separation diagnostic-only. The strict held-out
separator has zero eligible families and zero applications across the
four-repo recent-to-older proof. Do not promote runtime/default semantic
retention. The next semantic R&D branch, if continued, needs either a larger
training surface with the same held-out bar or a different source-free feature
set; do not relax eligibility inside the same proof just to force applications.
