# Phase 285 Broader Learned Policy Train/Test Surface

## Scope

Measure whether Phase 284's negative learned-policy train/test result is only a
small-slice power problem. This phase uses the existing source-free
`ctxhelm eval learned-policy-train-test` evaluator with broader disjoint
recent-to-older slices before adding any new backoff or aggregation rule.

## Pre-Registered Bar

Treat the broader proof as promotion-blocked unless at least one measured repo
has all of the following:

- `trainTestEligibleProfileOverlapCount > 0`
- `appliedCommitCount > 0`
- `targetHitDelta > 0`
- `regressedCommitCount = 0`
- `sourceTextLogged = false`
- `runtimePromotable = false`

If broader slices still produce no eligible overlap or no applications, record
the result as statistical-surface evidence and move next to pre-registered
backoff/cross-repo aggregation or semantic query/document construction.

## Planned Commands

Use the same stable source-free recent-to-older ranges from Phase 284, but
increase both train and test limits to `40`:

```bash
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --train-base ac1b3010505e --train-head 949bddcd3509 --test-base daaa4cc7bc5a --test-head 0f8e58c5fc16 --train-limit 40 --test-limit 40 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase285-train-test-refactoringminer-recent-to-older-limit40.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --train-base 7ce785776f70 --train-head 7f439fa1b1f2 --test-base fd6306d9788c --test-head e25092beaf56 --train-limit 40 --test-limit 40 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase285-train-test-ctxhelm-recent-to-older-limit40.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --train-base 1655371d2d31 --train-head 44277ff7d89a --test-base 04f5dd66f1ae --test-head 59fb828ffdb4 --train-limit 40 --test-limit 40 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase285-train-test-reagent-recent-to-older-limit40.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --train-base 1fb0027efe4b --train-head 335786673044 --test-base 6dd8f127cdbd --test-head 85920ebe534c --train-limit 40 --test-limit 40 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase285-train-test-verischema-recent-to-older-limit40.json
```

## Results

The requested `40`/`40` limit expanded only the repositories whose stable range
had enough usable commits. ctxhelm, ReAgent, and RefactoringMiner each exposed
only `25` train commits in the selected recent range; VeriSchema exposed `38`
train commits and `39` test commits.

| Repo | Train commits | Test commits | Eligible train profiles | Support histogram | Test candidate profiles | Train/test overlap | Eligible overlap | Applied commits | Target delta | Regressions | Decision |
| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| ctxhelm | 25 | 20 | 0 | `1=1, 2=2, 3=1, 4=1, 6=1, 7=1, 9=1, 11=1, 12=1` | 15 | 9 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |
| ReAgent | 25 | 20 | 0 | `1=7, 2=1, 3=2, 4=1, 5=1, 11=2` | 14 | 8 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |
| RefactoringMiner | 25 | 20 | 1 | `1=6, 2=3, 4=1, 6=1, 12=1` | 8 | 6 | 0 | 0 | 0 | 0 | `insufficient_test_applications` |
| VeriSchema | 38 | 39 | 0 | `1=1, 2=2, 4=1, 5=2, 6=1, 8=1, 9=1, 16=1` | 6 | 5 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |

## Interpretation

This remains an empty train/test result under the pre-registered bar. It is not
a no-regress win because `appliedCommitCount = 0` everywhere. The source-free
privacy and promotion invariants still hold (`sourceTextLogged = false`,
`runtimePromotable = false`), but they are degenerate in an empty application
set.

The funnel points to the eligibility gate and eligible-overlap gate rather than
simple `limit 20` starvation:

- ctxhelm, ReAgent, and VeriSchema have repeated train profiles, but all
  repeated profiles are blocked by inserted semantic non-targets, missing
  inserted target hits, or lost default targets.
- RefactoringMiner again trains the only eligible profile
  (`symbol_identifier/docs`, observed in two commits), but it has no eligible
  overlap with older test candidate profiles and therefore no applications.
- VeriSchema's deeper `38`/`39` split still has zero eligible train profiles, so
  its current semantic problem remains query/document or candidate-quality
  oriented rather than a small-slice learned-policy holdout issue.

Claude Code `2.1.163` gave a no-tool critique before the run: the `40`/`40`
probe is defensible only if the funnel counts are recorded, and an empty result
should be classified as inconclusive/no-signal rather than as a pass. This note
uses that framing.

## Decision

Stop doubling these same four stable ranges. The next learned-policy step must
either pre-register a backoff/cross-repo aggregation rule and measure it with
the same train/test funnel, or move back to semantic query/document construction
where the reports show no safe eligible train profiles.

## Validation

```bash
bash scripts/check-release-docs.sh
git diff --check
```
