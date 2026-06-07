# Phase 292 - Candidate-Path Concept Semantic Query Hints

## Scope

Phase 291 rejected more path-neighborhood aliases. Phase 292 tests a different
source-free query-construction idea: keep candidate-path aliases, then append a
small bounded set of generic software-domain concept terms derived from the task
and top lexical candidate path terms.

The eval-only mode is:

```bash
--semantic-query-mode candidate-path-concept-hints
```

The concept map is generic and source-free. Examples include:

- `verification` / `verifier` / `z3` -> `formal`, `solver`, `constraint`;
- `workflow` -> `state`, `transition`, `orchestration`;
- `gate` -> `validation`, `guard`, `check`;
- `prompt` -> `template`, `instruction`.

The mode is capped to four added concept tokens and does not change
runtime/default query behavior.

Claude Code `opus` was retried for a second opinion, but the local CLI reported
`Not logged in`, so no external recommendation was used.

## Pre-Registered Bar

Treat this as useful query-construction evidence only if it improves the
targeted VeriSchema older-range proof without selection or tail-slot
regressions:

- semantic candidate missed targets fall below Phase 289's `3`;
- selected semantic target hits stay at or above Phase 289's `11`;
- semantic-only non-targets do not rise materially;
- `semantic_corroborated_reranked` no longer regresses, or
  `semantic_tail_slot_reranked` stays neutral while adding target hits.

## Proof Command

```bash
cargo build -p ctxhelm --features local-embeddings --locked

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-query-mode candidate-path-concept-hints \
  --format json \
  > .ctxhelm/e2e/phase292-candidate-path-concept-hints-verischema-older-limit20.json
```

## Results

| Mode | Semantic candidate targets | Candidate misses | Selected semantic targets | Semantic-only targets | Semantic-only non-targets | Corroborated target-hit delta | Tail-slot target-hit delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `candidate-path-hints` | `14` | `3` | `11` | `2` | `16` | `-1` | `0` |
| `candidate-sibling-path-hints` | `14` | `4` | `10` | `2` | `15` | `-1` | `0` |
| `candidate-path-concept-hints` | `13` | `4` | `9` | `2` | `19` | `-1` | `-1` |

The concept mode worsens both candidate generation and selected semantic
quality relative to Phase 289. It also regresses the conservative tail-slot
variant: `semantic_tail_slot_reranked` drops to Recall@10 `0.2736842` and loses
`tests/agents/test_relationship_cardinality.py`.

## Decision

Reject candidate-path concept hints as a semantic promotion path. Generic
software-domain concepts are too noisy for the current local-fastembed query
shape on this proof slice.

The next semantic R&D should stop adding terms to a single semantic query. The
remaining options are a materially different source-free document construction
experiment or a narrower learned separator with held-out no-regress evidence.
