# Phase 287 - Source-Role Semantic Query Hints

## Scope

Phase 286 rejected profile-key relaxation, so Phase 287 moved to semantic query construction. The tested hypothesis was narrow and source-free: for explicit historical/eval semantic runs, append generic coding-role terms plus the dominant source language(s) to the semantic query. This is exposed only as an eval option:

```bash
--semantic-query-mode source-role-hints
```

Runtime/default behavior remains `plain`.

## Pre-Registered Bar

Treat this as useful only if it improves the targeted VeriSchema Python-source candidate quality without increasing regressions:

- semantic candidate target hits increase;
- semantic-only target hits do not fall;
- semantic-only non-targets do not rise materially;
- `semantic_corroborated_reranked` has no target-hit regression.

## Proof Commands

```bash
cargo build -p ctxhelm --features local-embeddings --locked

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --format json \
  > .ctxhelm/e2e/phase287-plain-verischema-older-limit20.json

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-query-mode source-role-hints \
  --format json \
  > .ctxhelm/e2e/phase287-source-role-hints-verischema-older-limit20.json
```

## Results

| Mode | Semantic candidate targets | Candidate misses | Semantic selected targets | Semantic-only targets | Semantic-only non-targets | Corroborated target-hit delta | Regressed commits |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `plain` | 13 | 4 | 9 | 2 | 19 | -1 | 1 |
| `source-role-hints` | 14 | 6 | 8 | 0 | 14 | -1 | 1 |

Variant recall did not improve:

| Mode | `ctxhelm_default` Recall@10 | `local_semantic` Recall@10 | `semantic_corroborated_reranked` Recall@10 |
| --- | ---: | ---: | ---: |
| `plain` | 0.29122806 | 0.29122806 | 0.28245613 |
| `source-role-hints` | 0.29122806 | 0.29122806 | 0.28245613 |

The source-role query added one candidate target, but it also increased candidate misses, reduced selected semantic target hits, removed the two plain-mode semantic-only target hits, and preserved the same semantic-corroborated regression.

## Decision

Reject `source-role-hints` as a semantic promotion path. Keep the mode as an eval-only diagnostic knob because it makes query-construction experiments reproducible and cache-distinct, but do not use it for runtime/default policy.

The next semantic branch should not add generic source-role words. It needs either richer task-specific semantic document/query construction or a cross-repo learned rule with a no-regress held-out bar.
