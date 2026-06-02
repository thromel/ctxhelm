# Phase 181: Graph Edge Budget Allocation

## Goal

Use the Phase 179/180 graph diagnostics to make a small ranking improvement
rather than only measuring graph pressure. Phase 180 showed that `imports`
produced some unique target lift on VeriSchema while `python_reexport` added
candidate/selection pressure without exclusive top-10 target lift on the fixed
fixture.

## Change

`source_dependency_floor` now allocates tight dependency-source budget by graph
edge family before raw dependency score:

1. `precision:*`
2. `imports`
3. other dependency edge labels
4. `python_reexport`

The change is bounded to source dependency floor ordering. It does not increase
context budget, recurse graph expansion, add new edge types, or alter lexical,
symbol, test, history, semantic, memory, or context-area channels.

## Proof

Focused tests:

```bash
cargo test -p ctxhelm-compiler --locked dependency_budget -- --nocapture
cargo test -p ctxhelm-compiler --locked selection_reserves_dependency -- --nocapture
```

Results:

- `selection_allocates_dependency_budget_by_edge_family` passed.
- Existing dependency reserve tests passed.

Fixture hydration:

```bash
bash scripts/prepare-proof-fixtures.sh
```

RefactoringMiner, ctxhelm, and ReAgent hydrated to the fixed revisions. The
fixed VeriSchema revision `b5cfb2a551d026514f505c45863db31277bcd1ad` is no
longer available from `git@github.com:thromel/VeriSchema.git`; GitHub API also
returns no commit for that SHA. This means the original four-repo Phase 110
fixture cannot be reproduced from a fresh clone without an external object
backup.

Three-fixture proof command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase181-graph-budget-three-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config /tmp/ctxhelm-rd/phase181-three-fixture-config.json \
  --format json > /tmp/ctxhelm-rd/phase181-graph-edge-budget-three-warm-proof.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase181-graph-edge-budget-three-warm-proof.json
```

Warm result:

- `releaseGate.decision = promote`
- Average File Recall@10: ctxhelm `0.75555557` vs lexical `0.5687831`
- Average lift at 10: `+0.18677251`
- `allFileClaim = mixed`
- `allFileBeatCount = 2`
- `allFileTrailCount = 1`
- `allFileExplainedTrailCount = 1`
- `allFileUnexplainedTrailCount = 0`

Cold result:

- The same three-fixture proof blocked only because RefactoringMiner exceeded
  the `5000ms` per-commit runtime ceiling.

Current reachable VeriSchema probe:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase181-current-verischema-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config /tmp/ctxhelm-rd/phase181-current-verischema-config.json \
  --format json > /tmp/ctxhelm-rd/phase181-current-verischema-proof.json
```

Result:

- `releaseGate.decision = promote`
- Current VeriSchema File Recall@10: ctxhelm `0.35529414` vs lexical
  `0.21176472`
- Average lift at 10: `+0.14352942`
- `imports`: `91` candidates, `20` selected@10, `18` retrieval targets,
  `5` target hits@10, `13` target misses@10
- `python_reexport`: `14` candidates, `1` selected@10, `0` retrieval targets,
  `0` target hits@10

## Interpretation

This is a bounded GraphRAG ranking change based on measured edge-family
behavior. The currently reproducible fixtures promote, and the current
VeriSchema probe confirms `python_reexport` remains pressure without target
lift. The missing pinned VeriSchema object should be treated as a fixture
freshness problem for future proof infrastructure, not as a reason to avoid the
ranking fix.
