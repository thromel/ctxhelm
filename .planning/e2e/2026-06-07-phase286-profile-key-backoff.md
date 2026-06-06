# Phase 286 Learned Policy Profile-Key Backoff

## Scope

Add an eval-only profile-key mode to the learned semantic policy train/test
evaluator. The default remains the strict `(query_family, path_family)` key.
The new `path_family_backoff` mode aggregates training evidence by path family
with `queryFamily = "*"`, applies the same support and zero-harm gates, and
records per-query-family breakdown rows for each learned profile.

## Pre-Registered Bar

Treat a backoff result as useful R&D signal only when a disjoint test report has
all of the following:

- `profileKeyMode = "path_family_backoff"`
- `trainTestEligibleProfileOverlapCount > 0`
- `appliedCommitCount > 0`
- `targetHitDelta > 0`
- `regressedCommitCount = 0`
- `sourceTextLogged = false`
- `runtimePromotable = false`

If a backoff mode applies but has no lift, or applies with regression, keep it
diagnostic-only and do not use it as runtime/default evidence.

## Validation Commands

```bash
cargo test -p ctxhelm-compiler learned_policy_train_test --locked
cargo test -p ctxhelm learned_policy_train_test_command_parses_disjoint_ranges --locked
cargo run -q -p ctxhelm -- eval learned-policy-train-test --help
```

## Planned Proof Commands

Use the same stable recent-to-older ranges from Phases 284/285, with
`--profile-key-mode path-family-backoff`:

```bash
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --train-base ac1b3010505e --train-head 949bddcd3509 --test-base daaa4cc7bc5a --test-head 0f8e58c5fc16 --train-limit 40 --test-limit 40 --budget 10 --semantic-provider local_fastembed --profile-key-mode path-family-backoff --format json > .ctxhelm/e2e/phase286-backoff-refactoringminer-recent-to-older-limit40.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --train-base 7ce785776f70 --train-head 7f439fa1b1f2 --test-base fd6306d9788c --test-head e25092beaf56 --train-limit 40 --test-limit 40 --budget 10 --semantic-provider local_fastembed --profile-key-mode path-family-backoff --format json > .ctxhelm/e2e/phase286-backoff-ctxhelm-recent-to-older-limit40.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --train-base 1655371d2d31 --train-head 44277ff7d89a --test-base 04f5dd66f1ae --test-head 59fb828ffdb4 --train-limit 40 --test-limit 40 --budget 10 --semantic-provider local_fastembed --profile-key-mode path-family-backoff --format json > .ctxhelm/e2e/phase286-backoff-reagent-recent-to-older-limit40.json
./target/debug/ctxhelm eval learned-policy-train-test --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --train-base 1fb0027efe4b --train-head 335786673044 --test-base 6dd8f127cdbd --test-head 85920ebe534c --train-limit 40 --test-limit 40 --budget 10 --semantic-provider local_fastembed --profile-key-mode path-family-backoff --format json > .ctxhelm/e2e/phase286-backoff-verischema-recent-to-older-limit40.json
```

## Results

| Repo | Mode | Train commits | Test commits | Eligible profiles | Support histogram | Test candidate profiles | Train/test overlap | Eligible overlap | Applied commits | Target delta | Regressions | Decision |
| --- | --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| ctxhelm | `path_family_backoff` | 25 | 20 | 0 | `3=1, 8=1, 21=1, 25=1` | 5 | 4 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |
| ReAgent | `path_family_backoff` | 25 | 20 | 0 | `1=2, 3=1, 9=1, 16=2` | 5 | 5 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |
| RefactoringMiner | `path_family_backoff` | 25 | 20 | 0 | `1=1, 2=1, 3=1, 9=1, 19=1` | 3 | 3 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |
| VeriSchema | `path_family_backoff` | 38 | 39 | 0 | `2=1, 11=1, 16=1, 29=1` | 4 | 3 | 0 | 0 | 0 | 0 | `insufficient_train_profiles` |

## Interpretation

Path-family backoff is rejected for promotion evidence. It increases coarse
train/test profile overlap, but every repeated path-family aggregate is blocked
before application:

- ctxhelm repeated `docs`, `planning`, `rust_source`, and `scripts`, but all
  had inserted semantic non-targets or lost default targets in the query-family
  breakdown.
- ReAgent repeated `docs` and `planning` heavily, but `docs` had no inserted
  semantic target hits and `planning` had inserted non-targets.
- RefactoringMiner's strict eligible `symbol_identifier/docs` row is swallowed
  by path-family aggregation because `commit_clue/docs` contributes non-target
  insertions, making `*/docs` unsafe.
- VeriSchema's repeated `docs` and `python_source` path families have no
  inserted semantic target hits, while `other` and `scripts` are blocked by
  inserted non-targets or lost default targets.

The per-query-family breakdown is doing the intended safety work: it explains
why a coarser key would hide query-specific noise if the zero-harm gate were
relaxed.

## Decision

Keep `path_family_backoff` as an eval-only diagnostic mode. Do not promote it
or use it as runtime/default behavior. The next semantic R&D branch should move
away from profile-key relaxation and toward semantic query/document
construction or a richer cross-repo rule that does not collapse known noisy
query families into broad path buckets.
