# Semantic Ablation Summary

## Setup

- Repository: `RefactoringMiner-clean`
- Head: `949bddcd3509a805f5e3bcc55fcdb71a691b0dac`
- Evaluated commits: `20`
- Ranking budget: `10`
- Mode: `bug_fix`
- Target agent: `claude-code`
- Privacy: local-only, source-free reports

## Variants

| Variant | semanticEnabled | fileRecallAt10 | lexicalBaselineRecallAt10 | ctxhelmLiftAt10 | testRecallAt10 | MRR@10 | runtime.totalMillis |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Default | `false` | `0.518631` | `0.5007738` | `0.017857194` | `0.4722222` | `0.6375` | `84398` |
| Local semantic | `true` | `0.518631` | `0.5007738` | `0.017857194` | `0.4722222` | `0.6350` | `104901` |

## Deltas

- File Recall@10 delta: `+0.000000`
- Test Recall@10 delta: `+0.000000`
- MRR@10 delta: `-0.0025`
- Runtime delta: `+20503ms` (`+24.29%`)
- Commits whose recommended context file set changed: `14/20`
- Commits with improved file hits: `0/20`
- Commits with worsened file hits: `0/20`
- Semantic-only signal changed-file hits: `0/82`
- Semantic-only signal recall: `0.0000`

## Interpretation

The current local semantic provider did not improve RefactoringMiner retrieval quality. It changed many recommendations, but none of those changes recovered additional gold changed files. The extra candidates were neutral for Recall@10, slightly negative for ranking order, and more expensive.

This does not mean semantic retrieval is useless as a product direction. It means the current `local_hash` provider is only a privacy-safe contract/provenance implementation, not a strong code retrieval backend. It should remain opt-in and should not be presented as a quality improvement for this corpus.

## Recommended Follow-Up

1. Keep `semanticEnabled` disabled by default.
2. Treat `local_hash` semantic retrieval as a deterministic test provider.
3. Add a stronger local embedding backend or optional code embedding backend behind explicit policy gates.
4. Add semantic quality gates requiring positive Recall@10 or MRR lift before enabling semantic in benchmark/release claims.
5. Focus near-term retrieval-quality work on the dominant gap families: `no_candidate_signal` for RefactoringMiner MCP source/test files and `ranked_below_budget_related_test`.
