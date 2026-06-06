# Phase 282 - Broader Learned Policy Holdout Slice

## Goal

Test whether Phase 281's zero-application holdout result was only a small-slice
artifact. The probe keeps the same source-free learned policy contract and
raises the historical gate window from `limit 20` to `limit 40`.

## Commands

```bash
cargo build -p ctxhelm --features local-embeddings
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 40 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase282-learned-policy-holdout-limit40-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 40 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase282-learned-policy-holdout-limit40-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 40 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase282-learned-policy-holdout-limit40-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 40 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase282-learned-policy-holdout-limit40-verischema.json
```

## Results

| Repo | Evaluated commits | Default | Semantic-corroborated | Learned-profile | Holdout | Policy profiles | Eligible profiles | Holdout applied commits | Holdout decision |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | `40` | `0.38511905` | `0.44845238` | `0.43511906` | `0.38511905` | `14` | `1` | `0` | `insufficient_eligible_holdout_profiles` |
| ctxhelm | `40` | `0.3893525` | `0.4548994` | `0.3893525` | `0.3893525` | `16` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| ReAgent | `40` | `0.37554947` | `0.35132784` | `0.37554947` | `0.37554947` | `19` | `0` | `0` | `insufficient_eligible_holdout_profiles` |
| VeriSchema | `38` | `0.34148607` | `0.35505673` | `0.34148607` | `0.34148607` | `10` | `0` | `0` | `insufficient_eligible_holdout_profiles` |

RefactoringMiner still exports only one eligible durable policy row:
`symbol_identifier/docs`, observed in `2` commits with `2` inserted targets,
`0` inserted non-targets, and `0` lost default targets. That is enough for the
full-snapshot artifact but not enough for leave-one-out application: when one
of those two supporting commits is held out, the remaining support drops below
`minimumSupportCommitCount = 2`.

## Claude Availability Check

Claude Code `2.1.163` is installed, but a bounded no-tool preflight timed out
after `45` seconds without emitting stream events. No raw prompt, transcript,
or MCP traffic was written to the repository. This means no Claude opinion was
available for this phase; it is client availability evidence, not retrieval
quality evidence.

## Decision

Keep runtime/default semantic promotion blocked.

The broader slice rules out the simplest "just use more commits in the same
four-repo gate" explanation for Phase 281. The next learned-policy R&D needs a
different training/holdout design: either explicit train/test revision ranges,
more repositories with repeated profile families, or a policy artifact that can
aggregate profile evidence across compatible repos while keeping source-free
staleness and no-regress gates.
