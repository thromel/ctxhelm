# Phase 72 Broader Repeated-Lift Validation

Date: 2026-05-30

## Goal

Validate whether the promoted two-repo retrieval policy generalizes to more local repositories, then fix any concrete production gap found during the proof.

## Change

- Increased the `prepare_task` related-test selection budget from 5 to 10 so the plan output can satisfy Test Recall@10.
- Seeded related-test discovery from co-changed and dependency-neighbor source files, not only the initial lexical/symbol/semantic source seeds.
- Kept ctxhelm read-only and source-free; no source text is logged in proof output.

## Proof Commands

```bash
cargo test -p ctxhelm-compiler prepare_context_plan_ -- --nocapture
cargo run -p ctxhelm -- eval proof --config /tmp/ctxhelm-phase72-broad-repeated-lift-config.json --format json > /tmp/ctxhelm-phase72-broad-repeated-lift-proof-after-test-seeds.json
cargo run -p ctxhelm -- eval proof --config .ctxhelm/e2e/v25-multirepo-baseline-config.json --format json > /tmp/ctxhelm-current-two-repo-proof-after-test-seeds.json
```

## Required Two-Repo Gate

The fixed v2.5 two-repo gate still promotes default local retrieval after the test-selection changes.

| Corpus | Status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected Miss-Rate@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | beat | 0.7778 | 0.7407 | 1.0000 | 0.0526 |
| ctxhelm | beat | 0.5063 | 0.4684 | 1.0000 | 0.1429 |

`releaseGate.decision = promote`.

## Broader Four-Repo Probe

The broader optional probe used RefactoringMiner, ctxhelm, ReAgent, and VeriSchema with `limit = 5` per repository. It is intentionally not a release gate yet because the added corpora are not fixed committed benchmark fixtures.

| Corpus | Status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected Miss-Rate@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | match | 0.7000 | 0.7000 | 1.0000 | 0.0652 |
| ctxhelm | beat | 0.3611 | 0.3056 | 0.0000 | 0.1633 |
| ReAgent | beat | 0.7143 | 0.5714 | 1.0000 | 0.2174 |
| VeriSchema | trail | 0.1507 | 0.0822 | 0.6614 | 0.1163 |

`releaseGate.decision = block` on the broader probe because RefactoringMiner matched rather than beat and VeriSchema still missed the 0.80 validation-test floor.

## Measured Improvement

On VeriSchema, Test Recall@10 moved through these measurements:

- Before changes: 0.2434
- Related-test budget increased to 10: 0.3280
- Related-test seeds expanded with co-change/dependency source neighbors: 0.6614

A test-directory diversity experiment was tried and rejected because it reduced VeriSchema Test Recall@10 to 0.4921.

## Remaining Gaps

- VeriSchema still trails the broader gate because validation-test recall is below 0.80 on large multi-area commits.
- Protected evidence miss-rate remains non-zero on every corpus.
- RefactoringMiner can match instead of beat when the broader probe uses only the newest 5 commits.
- The broad proof should be converted into a committed fixed-corpus benchmark only after choosing stable repo/revision ranges.
