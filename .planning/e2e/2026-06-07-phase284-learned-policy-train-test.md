# Phase 284 Learned Policy Train/Test

## Scope

Add a source-free train/test evaluator for the learned semantic policy path.
The evaluator trains repeated-support `(query_family, path_family)` profile
eligibility on one revision range and applies the resulting policy to a
disjoint test range. It keeps semantic policy promotion blocked by reporting
`runtimePromotable = false`.

## Code Changes

- Added `LearnedSemanticPolicyTrainTestReport`.
- Added `learned_semantic_policy_train_test_report_with_provider_and_ranges`.
- Added `ctxhelm eval learned-policy-train-test`.
- Added support diagnostics for negative/empty results:
  `trainProfileSupportHistogram`, `testCandidateProfileCount`,
  `trainTestProfileOverlapCount`, and
  `trainTestEligibleProfileOverlapCount`.

## Validation Commands

```bash
cargo test -p ctxhelm-compiler learned_policy_train_test_applies_training_profiles_to_disjoint_test_range --locked
cargo test -p ctxhelm learned_policy_train_test_command_parses_disjoint_ranges --locked
cargo fmt --check
cargo test -p ctxhelm-compiler semantic --locked
cargo run -q -p ctxhelm -- eval learned-policy-train-test --help
cargo build -p ctxhelm --features local-embeddings
```

## Local-Fastembed Train/Test Proof

All reports use recent slices for training and older slices for disjoint test
application:

```bash
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --train-base ac1b3010505e --train-head 949bddcd3509 --test-base daaa4cc7bc5a --test-head 0f8e58c5fc16 --train-limit 20 --test-limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase284-train-test-refactoringminer-recent-to-older.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --train-base 7ce785776f70 --train-head 7f439fa1b1f2 --test-base fd6306d9788c --test-head e25092beaf56 --train-limit 20 --test-limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase284-train-test-ctxhelm-recent-to-older.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --train-base 1655371d2d31 --train-head 44277ff7d89a --test-base 04f5dd66f1ae --test-head 59fb828ffdb4 --train-limit 20 --test-limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase284-train-test-reagent-recent-to-older.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --train-base 1fb0027efe4b --train-head 335786673044 --test-base 6dd8f127cdbd --test-head 85920ebe534c --train-limit 20 --test-limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase284-train-test-verischema-recent-to-older.json
```

| Repo | Train commits | Test commits | Eligible train profiles | Support histogram | Test candidate profiles | Train/test overlap | Eligible overlap | Applied commits | Target delta | Regressions | Decision |
| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| ctxhelm | 20 | 20 | 0 | `1=3, 2=1, 3=1, 5=1, 6=2, 11=2` | 15 | 9 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |
| ReAgent | 20 | 20 | 0 | `1=4, 2=1, 3=2, 4=1, 5=1, 9=1, 11=1` | 14 | 5 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |
| RefactoringMiner | 20 | 20 | 1 | `1=5, 2=2, 3=2, 9=1` | 8 | 5 | 0 | 0 | 0 | 0 | `insufficient_test_applications` |
| VeriSchema | 20 | 19 | 0 | `1=1, 2=3, 3=3, 5=1, 6=1, 7=1` | 5 | 5 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |

## Interpretation

This is a negative R&D result. The evaluator now measures true disjoint
train/test application, but the current learned-policy signal has no
eligible-profile test overlap and zero external applications. Repeated profile
support exists in the training slices, but most repeated rows are blocked by
inserted non-targets, no inserted target hits, or lost default targets.
RefactoringMiner trains the only eligible row (`symbol_identifier/docs`) but it
does not overlap the older test candidate profiles.

Claude Code `2.1.163` was available for a no-tool design critique. The useful
recommendation was to treat this as a sparsity/power result, not a failed
policy proof: report support distributions and train/test overlap before
concluding the learned policy has no effect. The implementation therefore
keeps those diagnostics in the train/test report.

## Decision

Keep learned semantic policy eval-only. Do not promote semantic reranking or
runtime defaults. The next policy experiment should pre-register a larger
statistical surface, backoff hierarchy, or cross-repo aggregation rule, then
measure it with the same source-free train/test report and no-regress bar.
