# Phase 203 - Related Test Evidence Pack Section

## Goal

Close the Phase 201/202 validation-consumption gap in generated packs. The proof
showed that some files missed by top-10 context are already recoverable through
the broader agent evidence bundle, especially selected related tests. Those
tests should be visible where an agent consumes the pack, not only in JSON
metrics or command lists.

## Change

Generated packs now include a source-free `Related test evidence` section when
related tests are selected. The section lists:

- test path
- context area
- selection reason
- confidence
- targeted command when available

The section also explicitly says selected validation evidence may not be
repeated in context-area next-read lists. This preserves the distinction between
selected validation evidence and progressive unread area guidance.

## Validation

Focused command:

```bash
cargo test -p ctxhelm-compiler \
  compile_context_pack_materializes_plan_snippets_and_validation \
  --locked -- --nocapture
```

Result: pass.

Product proof command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase203-related-test-evidence-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase203-related-test-evidence-proof.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase203-related-test-evidence-proof.json
```

Result: `releaseGate.decision = promote`.

Metrics remain unchanged from Phase 201:

- Average File Recall@10: `0.7082679`
- Average lexical File Recall@10: `0.4795285`
- Average file delta vs lexical: `+0.22873944`
- Average Agent Evidence Recall@10: `0.81052285`
- Average Context Recall@10: `0.7708334`
- All-file beat/match/trail: `3 / 0 / 1`
- Agent-evidence beat/match/trail: `3 / 1 / 0`
- Context beat/match/trail: `3 / 1 / 0`

The focused test now asserts that generated markdown includes:

- `Related test evidence`
- `Area: tests/auth`
- the selected related test path
- the targeted validation command
- the note about selected validation evidence not necessarily being repeated in
  context-area next-read lists

## Interpretation

This phase does not claim retrieval lift. It improves agent consumption of
evidence that ctxhelm already selects and that product proof already counts as
agent evidence. The next proof-oriented R&D loop should measure whether real
agent runs consume this section more reliably than the prior generic checklist.
