# Phase 184: Context Area Signal Profiles

## Goal

Make progressive broad-area guidance more explainable after the Phase 183 proof
showed the remaining gaps are often budget pressure inside already-surfaced
areas, not simply missing area discovery.

The product need is agent-facing: when ctxhelm tells an agent to inspect a
context area next, the agent should see whether that area is being surfaced by
lexical, dependency, co-change, semantic, test, docs, config, anchor, or memory
signals. That makes follow-up native reads more deliberate without forcing more
files into the protected top-10 budget.

## Gap Analysis

Fresh Phase 183 clean fixture proof:

```text
/tmp/ctxhelm-rd/phase183-clean-fixture-refresh-proof-scoped.json
```

Observed corpus metrics:

| Corpus | File Recall@10 | Lexical File Recall@10 | Source Recall@10 | Test Recall@10 | Effective Validation |
| --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 0.8000000 | 1.0000000 | 1.0000000 | 1.0000000 | 1.0000000 |
| ctxhelm | 0.6666667 | 0.3063492 | 0.4666667 | n/a | n/a |
| ReAgent | 0.8000000 | 0.4000000 | 1.0000000 | 1.0000000 | 1.0000000 |
| VeriSchema | 0.3552941 | 0.2117647 | 0.5000000 | 0.7896825 | 1.0000000 |

The repeated VeriSchema misses were concentrated under areas such as
`schema_agent/agents`, `schema_agent/core`, and `schema_agent/nlp`, with gap
codes including:

- `ranked_below_budget_dependency`
- `ranked_below_budget_co_change`
- `area_context_only`

That means the system was often finding the right region but not always fitting
the exact files into the top-10 evidence budget.

## Rejected Experiment

I tested a bounded dependency reserve increase:

```rust
// temporary experiment only
file_budget.div_ceil(4).clamp(1, 3)
// to
file_budget.div_ceil(3).clamp(1, 4)
```

The focused test passed, but the four-repo proof showed no recall lift:

- File recall delta: `0`
- Source recall delta: `0`
- Test recall delta: `0`
- Effective validation delta: `0`
- Protected miss pressure worsened on some corpora.

The experiment was removed. This preserves the current top-10 ranking behavior
instead of taking a larger dependency budget that measured as neutral or worse.

## Implementation

- Added `signalCounts` to the public `ContextArea` contract.
- Recorded one count per candidate per signal family, deduplicated within the
  candidate so a path with repeated lexical evidence does not inflate a whole
  area.
- Fell back from `signal_scores` to candidate evidence when the ranked
  candidate lacks explicit source scores.
- Rendered `Signals:` in context-area pack guidance next to role counts,
  selected role counts, representative paths, next reads, and resource URI.
- Covered the contract, planning, and pack rendering paths with focused tests.

Signal keys are source-free and stable:

```text
lexical
lexical_expansion
symbol
dependency
related_test
semantic
co_change
current_diff
history
docs
config
anchor
memory
```

## Prepare-Task Smoke

Command:

```bash
rm -rf /tmp/ctxhelm-phase184-signal-home
env CTXHELM_HOME=/tmp/ctxhelm-phase184-signal-home \
  cargo run -p ctxhelm --locked -- prepare-task \
  'update schema agent evaluation workflow' \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  > /tmp/ctxhelm-rd/phase184-verischema-prepare.out
```

Result:

- Prepare-task returned `16` context areas.
- `schema_agent/core` reported signals
  `co_change=1, dependency=3, lexical=3, lexical_expansion=4`, with `9`
  source-role candidates, `5` selected source candidates, and next reads such
  as `schema_agent/core/__init__.py`,
  `schema_agent/core/artifacts.py`, and
  `schema_agent/core/attribute_vocabulary.py`.
- `schema_agent/agents` reported signals
  `co_change=2, dependency=1, lexical_expansion=4`, with `5` source-role
  candidates, `1` selected source candidate, and next reads such as
  `schema_agent/agents/relationship_cardinality.py`,
  `schema_agent/agents/__init__.py`, and
  `schema_agent/agents/conceptual_model_reviewer.py`.
- `schema_agent/evaluation` reported signals
  `lexical=1, lexical_expansion=4`.

## Product Proof

Command:

```bash
rm -rf /tmp/ctxhelm-phase184-signal-proof-home \
  /tmp/ctxhelm-rd/phase184-context-area-signal-proof.json
env CTXHELM_HOME=/tmp/ctxhelm-phase184-signal-proof-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase184-context-area-signal-proof.json
python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase184-context-area-signal-proof.json
```

Result:

- `releaseGate.decision = promote`
- `143` historical context-area rows contained non-empty `signalCounts`.
- Sample context-area signal profiles:
  - RefactoringMiner `src/main/java/org/refactoringminer/mcp`:
    `co_change=7, lexical=4, lexical_expansion=1`
  - ctxhelm `crates/ctxpack-index`:
    `co_change=5, dependency=1, lexical=1`
  - ctxhelm `crates/ctxpack-compiler`:
    `co_change=7, dependency=1, lexical_expansion=1`

## Verification

Focused commands already passed before full workspace validation:

```bash
cargo test -p ctxhelm-core context_plan_public_json_shape_is_stable --locked -- --nocapture
cargo test -p ctxhelm-compiler context_areas_include_docs_and_next_read_paths --locked -- --nocapture
cargo test -p ctxhelm-compiler compile_context_pack_renders_context_areas --locked -- --nocapture
```

## Decision

Accept the source-free signal-profile instrumentation and pack rendering.
Reject the dependency-reserve widening until a later experiment can show
positive recall lift under the clean four-repo proof gate.
