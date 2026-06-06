# Phase 283 - Ranged Semantic Gate Slices

## Goal

Make explicit revision-slice semantic R&D reproducible. Phase 282 showed that
widening the same gate to `limit 40` still produced zero held-out learned-policy
applications. The next requirement is stable older/recent slice evidence, not
another implicit `HEAD` window.

## Code Kept

`ctxhelm eval gate` now accepts:

- `--base <REV>`: start revision for a stable semantic gate range
- `--head <REV>`: end revision for a stable semantic gate range

The compiler keeps the existing `semantic_precision_gate_report_with_provider`
API and adds a ranged entry point used by the CLI. All gate variants receive the
same `base` and `head`, so default, semantic, reranked, learned-profile, and
learned-policy holdout results remain comparable within a slice.

## Commands

```bash
cargo run -q -p ctxhelm -- eval gate --help
cargo test -p ctxhelm semantic_gate_command_parses_stable_revision_range --locked
cargo test -p ctxhelm-compiler semantic --locked
cargo build -p ctxhelm --features local-embeddings
```

Ranged probes:

```bash
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --base daaa4cc7bc5a --head 0f8e58c5fc16 --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase283-ranged-gate-refactoringminer-older.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --base ac1b3010505e --head 949bddcd3509 --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase283-ranged-gate-refactoringminer-recent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --base fd6306d9788c --head e25092beaf56 --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase283-ranged-gate-ctxhelm-older.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --base 7ce785776f70 --head 7f439fa1b1f2 --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase283-ranged-gate-ctxhelm-recent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --base 04f5dd66f1ae --head 59fb828ffdb4 --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase283-ranged-gate-reagent-older.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --base 1655371d2d31 --head 44277ff7d89a --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase283-ranged-gate-reagent-recent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --base 6dd8f127cdbd --head 85920ebe534c --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase283-ranged-gate-verischema-older.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --base 1fb0027efe4b --head 335786673044 --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase283-ranged-gate-verischema-recent.json
```

## Results

| Repo slice | Evaluated commits | Default | Semantic-corroborated | Learned-profile | Held-out policy | Eligible policy profiles | Holdout applied commits | Holdout decision |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner older | `20` | `0.38666666` | `0.35999998` | `0.38666666` | `0.38666666` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| RefactoringMiner recent | `20` | `0.41857147` | `0.5619047` | `0.51857144` | `0.41857147` | `1` | `0` | `insufficient_eligible_holdout_profiles` |
| ctxhelm older | `20` | `0.42655706` | `0.5038528` | `0.42655706` | `0.42655706` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| ctxhelm recent | `20` | `0.44620585` | `0.44333735` | `0.44620585` | `0.44620585` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| ReAgent older | `20` | `0.4635989` | `0.423489` | `0.4635989` | `0.4635989` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| ReAgent recent | `20` | `0.35` | `0.325` | `0.35` | `0.35` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| VeriSchema older | `19` | `0.29122806` | `0.28245613` | `0.29122806` | `0.29122806` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| VeriSchema recent | `20` | `0.39382353` | `0.36960787` | `0.39382353` | `0.39382353` | `0` | `0` | `insufficient_eligible_holdout_profiles` |

RefactoringMiner recent is still the only slice with an eligible durable policy
row: `symbol_identifier/docs`, observed in `2` commits. The older
RefactoringMiner slice has no matching eligible row, and no other repo slice has
eligible policy support.

## Decision

Keep the ranged gate CLI and source-free proof artifacts. The ranged probe is
negative for promotion: the current learned-policy signal does not repeat across
older/recent slices, and the only eligible row still cannot satisfy
leave-one-out support.

The next semantic learned-policy experiment should either aggregate compatible
profile evidence across repositories or introduce a true train-on-range /
apply-on-disjoint-test-range evaluator. Rerunning implicit `HEAD` windows is now
an exhausted path.
