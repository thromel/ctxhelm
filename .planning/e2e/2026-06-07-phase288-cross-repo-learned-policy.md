# Phase 288 - Cross-Repo Learned Policy Aggregation

## Scope

Phase 287 rejected generic source-role query hints, so Phase 288 tested the
other remaining learned-policy branch: source-free cross-repo aggregation of
saved `ctxhelm eval learned-policy-train-test` reports.

The new diagnostic command is report-only:

```bash
ctxhelm eval learned-policy-cross-repo \
  --report phase285-repo-a.json \
  --report phase285-repo-b.json \
  --minimum-repo-support 2 \
  --format json
```

It deserializes saved train/test JSON reports, aggregates profile keys across
repositories, and applies a zero-harm gate:

- profile appears in at least `minimumRepoSupportCount` repositories;
- aggregate inserted semantic target hits are positive;
- aggregate inserted semantic non-targets are zero;
- aggregate lost default targets are zero;
- `runtimePromotable` remains `false`.

## Proof Commands

Strict `(query_family, path_family)` Phase 285 reports:

```bash
./target/debug/ctxhelm eval learned-policy-cross-repo \
  --report .ctxhelm/e2e/phase285-train-test-ctxhelm-recent-to-older-limit40.json \
  --report .ctxhelm/e2e/phase285-train-test-reagent-recent-to-older-limit40.json \
  --report .ctxhelm/e2e/phase285-train-test-refactoringminer-recent-to-older-limit40.json \
  --report .ctxhelm/e2e/phase285-train-test-verischema-recent-to-older-limit40.json \
  --minimum-repo-support 2 \
  --format json \
  > .ctxhelm/e2e/phase288-cross-repo-learned-policy-phase285-limit40.json
```

Path-family backoff Phase 286 reports:

```bash
./target/debug/ctxhelm eval learned-policy-cross-repo \
  --report .ctxhelm/e2e/phase286-backoff-ctxhelm-recent-to-older-limit40.json \
  --report .ctxhelm/e2e/phase286-backoff-reagent-recent-to-older-limit40.json \
  --report .ctxhelm/e2e/phase286-backoff-refactoringminer-recent-to-older-limit40.json \
  --report .ctxhelm/e2e/phase286-backoff-verischema-recent-to-older-limit40.json \
  --minimum-repo-support 2 \
  --format json \
  > .ctxhelm/e2e/phase288-cross-repo-learned-policy-backoff-limit40.json
```

## Results

| Input reports | Profile mode | Source repos | Candidate profiles | Eligible profiles | Decision |
| --- | --- | ---: | ---: | ---: | --- |
| Phase 285 strict train/test | `query_path` | 4 | 28 | 0 | `insufficient_cross_repo_profiles` |
| Phase 286 path-family backoff | `path_family_backoff` | 4 | 9 | 0 | `insufficient_cross_repo_profiles` |

Most repeated strict keys are unsafe:

| Profile | Repos | Inserted targets | Inserted non-targets | Lost default targets | Decision |
| --- | ---: | ---: | ---: | ---: | --- |
| `symbol_identifier/docs` | 4 | 3 | 25 | 20 | non-target insertions |
| `broad_scope/docs` | 3 | 8 | 38 | 93 | non-target insertions |
| `domain_phrase/docs` | 3 | 5 | 72 | 12 | non-target insertions |
| `symbol_identifier/other` | 3 | 0 | 19 | 10 | no inserted target hits |
| `domain_phrase/planning` | 2 | 9 | 47 | 26 | non-target insertions |

The coarser path-family aggregates are worse:

| Profile | Repos | Inserted targets | Inserted non-targets | Lost default targets | Decision |
| --- | ---: | ---: | ---: | ---: | --- |
| `*/docs` | 4 | 16 | 140 | 125 | non-target insertions |
| `*/scripts` | 3 | 5 | 18 | 72 | non-target insertions |
| `*/other` | 3 | 2 | 32 | 11 | non-target insertions |
| `*/planning` | 2 | 19 | 86 | 106 | non-target insertions |
| `*/python_source` | 2 | 0 | 25 | 2 | no inserted target hits |

Both reports keep `sourceTextLogged = false`, local-only privacy, and
`runtimePromotable = false`.

## Decision

Reject current cross-repo learned-policy aggregation as a semantic promotion
path. The only previously clean strict row, RefactoringMiner
`symbol_identifier/docs`, becomes unsafe when the same key is aggregated across
other repositories. Coarse path-family aggregation is also rejected because it
hides much larger non-target insertion and lost-default-target pressure.

The remaining semantic R&D path should move to task-specific query/document
construction or a materially richer rule that can prove a narrower separator
before runtime/default promotion.
