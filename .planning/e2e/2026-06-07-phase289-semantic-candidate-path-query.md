# Phase 289 - Candidate-Path Semantic Query Hints

## Scope

Phase 288 rejected current cross-repo learned-policy aggregation, leaving
task-specific query/document construction as the next semantic branch. Phase
289 adds an eval-only semantic query construction probe:

```bash
--semantic-query-mode candidate-path-hints
```

The mode keeps default/runtime query behavior at `plain`. When enabled, the
planner runs lexical search first, extracts a bounded set of source-free alias
terms from the top lexical candidate paths, and appends those aliases to the
semantic query before semantic search. Generic scaffold terms such as `schema`,
`agent`, `tests`, `docs`, `source`, and `init` are excluded, and the hint set is
bounded by path count and term count.

## Pre-Registered Bar

Treat this as promotion evidence only if the targeted VeriSchema older-range
proof improves semantic candidate quality and converts it into selected recall:

- semantic candidate missed targets fall;
- selected semantic target hits rise;
- semantic-only target hits do not fall;
- semantic-only non-targets do not rise materially;
- `local_semantic` or `semantic_corroborated_reranked` recall improves without
  adding regressions.

If candidate quality improves but recall and the semantic-corroborated
regression remain unchanged, keep the mode diagnostic-only.

## Proof Commands

```bash
cargo build -p ctxhelm --features local-embeddings --locked

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-query-mode candidate-path-hints \
  --format json \
  > .ctxhelm/e2e/phase289-candidate-path-hints-verischema-older-limit20.json

./target/debug/ctxhelm eval history \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic --semantic-provider local_fastembed \
  --semantic-query-mode candidate-path-hints \
  --format json \
  > .ctxhelm/e2e/phase289-candidate-path-hints-history-verischema-older-limit20.json
```

Claude Code `2.1.163` was installed locally, but `claude --bare -p --model opus`
reported `Not logged in`, so no external second-opinion result was used.

## Results

| Mode | Semantic candidate targets | Candidate misses | Semantic selected targets | Semantic-only targets | Semantic-only non-targets | Corroborated target-hit delta | Regressed commits |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `plain` | 13 | 4 | 9 | 2 | 19 | -1 | 1 |
| `source-role-hints` | 14 | 6 | 8 | 0 | 14 | -1 | 1 |
| `candidate-path-hints` | 14 | 3 | 11 | 2 | 16 | -1 | 1 |

Variant recall still does not improve:

| Mode | `ctxhelm_default` Recall@10 | `local_semantic` Recall@10 | `semantic_corroborated_reranked` Recall@10 |
| --- | ---: | ---: | ---: |
| `candidate-path-hints` | 0.29122806 | 0.29122806 | 0.28245613 |

The matching history report confirms the mode changes semantic source/test/doc
selection on the missed commits, but the remaining misses still include
`schema_agent/core/state.py`, workflow/gate/prompt support files,
`schema_agent/agents/fd_extractor.py`, and prompt/verification files.

## Decision

Keep `candidate-path-hints` as an eval-only diagnostic. It is a better
candidate-generation probe than generic source-role words on the targeted
VeriSchema slice: candidate misses fall and selected semantic target hits rise.
It still does not clear the promotion bar because recall remains unchanged and
the semantic-corroborated regression remains.

The next semantic step should focus on converting improved semantic candidate
quality into target-preserving selection, or on richer document/query
construction for the remaining prompt/workflow/verification files. Do not treat
candidate-path hints as runtime/default evidence.
