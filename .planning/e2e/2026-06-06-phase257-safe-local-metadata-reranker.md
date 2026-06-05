# Phase 257: Safe Local Metadata Reranker

## Goal

Fix the eval-only `local_metadata_reranked` path so it can measure metadata
reranking lift without demoting protected anchor/current-diff/lexical/symbol
evidence or starving validation tests.

This follows Phase 256, where richer source-free semantic documents were
rejected. The remaining promising R&D path was the metadata reranker, but prior
gate reports kept it visible as unsafe because it could improve aggregate recall
while moving protected evidence out of the top-10 context budget.

## Change

- Preserve protected source evidence from the default top-K ranking before
  applying metadata reranking to the remaining file slots.
- Preserve the existing validation-test reserve behavior for narrow tasks.
- Keep the reranker policy-gated and eval-only; this does not enable reranking
  by default.
- Add focused tests for protected source preservation and test-reserve behavior.

## Proof

Artifacts:

- `.ctxhelm/e2e/phase257-reranker-protected-floor-refactoringminer.json`
- `.ctxhelm/e2e/phase257-reranker-protected-floor-ctxhelm.json`

Commands:

```bash
cargo test -p ctxhelm-compiler local_metadata --locked
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase257-reranker-protected-floor-refactoringminer.json
cargo run -q -p ctxhelm --features local-embeddings -- eval gate --repo /Users/romel/Documents/GitHub/ctxhelm --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase257-reranker-protected-floor-ctxhelm.json
```

RefactoringMiner:

| Variant | Recall@10 | Test Recall@10 | Protected Miss@10 | Protected Target Miss@10 | Named Regressions |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ctxhelm_default` | `0.5383333` | `1.0` | `0.35164836` | `0.125` | `0` |
| `local_metadata_reranked` | `0.5383333` | `1.0` | `0.20879121` | `0.125` | `0` |
| `local_semantic` | `0.5583333` | `1.0` | `0.34065935` | `0.125` | `0` |

ctxhelm:

| Variant | Recall@10 | Test Recall@10 | Protected Miss@10 | Protected Target Miss@10 | Named Regressions |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ctxhelm_default` | `0.30929655` | `1.0` | `0.5670103` | `0.6666667` | `0` |
| `local_metadata_reranked` | `0.5414069` | `1.0` | `0.15463917` | `0.15151516` | `0` |
| `local_semantic` | `0.30929655` | `1.0` | `0.5670103` | `0.6666667` | `0` |

## Decision

Promote the safety fix, not the default behavior.

The local metadata reranker is now safe enough to keep evaluating: it has zero
named regressions on both checked corpora, improves protected miss pressure, and
substantially improves ctxhelm Recall@10. It is still not a default promotion
because RefactoringMiner is neutral rather than improved and the overall gate
decision remains `hold`.

Next R&D should compare this safe reranker with query-family routing and learned
fusion. The useful path is no longer "can the reranker avoid demoting exact
evidence"; it can. The next question is when the reranker should run.
