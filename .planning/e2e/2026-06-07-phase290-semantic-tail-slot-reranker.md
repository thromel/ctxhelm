# Phase 290 - Tail-Slot Semantic Reranker

## Scope

Phase 289 showed that `candidate-path-hints` improved VeriSchema semantic
candidate quality, but the semantic-corroborated selector still lost the default
target `schema_agent/prompts/normalization.py` after non-target semantic files
entered the fixed top-10 budget.

Phase 290 adds the eval-only `semantic_tail_slot_reranked` gate variant. It
starts from the semantic-corroborated ranking, but preserves the top
`ceil(0.8 * K)` default files in order and lets semantic-corroborated candidates
compete only for the remaining tail slots. Runtime/default ranking is
unchanged.

## Pre-Registered Bar

Treat this as promotion evidence only if it both:

- removes the known semantic-corroborated/default-target regression; and
- converts candidate-path semantic candidates into positive target-hit delta
  without default-only target churn.

If it removes the regression but adds no target hits, keep it as a safety
diagnostic and continue query/document or learned-separator work.

## Proof Command

```bash
cargo build -p ctxhelm --features local-embeddings --locked

./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-query-mode candidate-path-hints \
  --format json \
  > .ctxhelm/e2e/phase290-tail-slot-semantic-verischema-older-limit20.json
```

## Results

`candidate-path-hints` still has the same candidate-quality counters as Phase
289:

- semantic candidate targets: `14`
- semantic candidate misses: `3`
- selected semantic targets: `11`
- semantic-only targets: `2`
- semantic-only non-targets: `16`

The tail-slot variant removes the measured semantic-corroborated regression but
does not create lift:

| Variant | Target-hit delta | Improved commits | Regressed commits | Default-only targets | Reranker-only targets |
| --- | ---: | ---: | ---: | ---: | ---: |
| `semantic_corroborated_reranked` | `-1` | `0` | `1` | `1` | `0` |
| `semantic_family_budget_reranked` | `-1` | `0` | `1` | `1` | `0` |
| `semantic_tail_slot_reranked` | `0` | `0` | `0` | `0` | `0` |

The new diagnostic emits `semantic_tail_slot_reranker_neutral`: it preserves
target hits, but it does not add target hits on this gate.

## Decision

Keep `semantic_tail_slot_reranked` eval-only and diagnostic. The result proves a
simple source-free high-confidence default-prefix guard can remove the current
VeriSchema semantic-corroborated regression, but the improved candidate-path
semantic pool still has no safe recall lift under the conservative tail budget.

The next semantic branch should not promote tail-slot semantic reranking. It
should move to richer query/document construction for the remaining
prompt/workflow/verification gaps, or to a narrower learned separator with
held-out no-regress evidence.
