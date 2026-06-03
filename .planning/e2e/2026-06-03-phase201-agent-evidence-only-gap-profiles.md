# Phase 201 - Agent Evidence Only Gap Profiles

## Goal

Explain the remaining gap between progressive context-area next reads and the
broader agent evidence bundle. Phase 200 improved selected-file recall, but the
proof still showed VeriSchema with many missed@10 files recoverable somewhere
outside selected top-10 context.

## Rejected Experiments

Three direct selection/read-order experiments were measured and rejected:

- Broad agent-source floor after source-history and before config/workflow.
- Role-aware `related_test` priority for validation next-read paths.
- Larger high-pressure context-area next-read cap.

All three passed focused tests or proof checks where applicable, but none moved
selected-file recall, next-read recovery, or agent-evidence recovery. They were
reverted rather than kept as dead heuristics.

## Accepted Change

`contextAreaNextReadSummary` now includes source-free profiles for files that
are recoverable through the full agent evidence bundle but absent from
progressive next-read paths:

- `agentEvidenceOnlyCount`
- `agentEvidenceOnlyRoleCounts`
- `topAgentEvidenceOnlyAreas`

This preserves source privacy: only roles, safe paths already present in eval
metadata, and context-area names are reported.

Focused validation:

```bash
cargo test -p ctxhelm-compiler eval::tests:: --locked -- --nocapture
```

Result: all eval tests pass, including assertions for test-only
agent-evidence-only gaps.

## Fresh Proof

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase201-agent-evidence-gap-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase201-agent-evidence-gap-proof.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase201-agent-evidence-gap-proof.json
```

Result: `releaseGate.decision = promote`.

## Metrics

Phase 201 keeps Phase 200 retrieval metrics unchanged:

- Average File Recall@10: `0.7082679`
- Average lexical File Recall@10: `0.4795285`
- Average file delta vs lexical: `+0.22873944`
- Average Agent Evidence Recall@10: `0.81052285`
- Average Context Recall@10: `0.7708334`
- All-file beat/match/trail: `3 / 0 / 1`
- Agent-evidence beat/match/trail: `3 / 1 / 0`
- Context beat/match/trail: `3 / 1 / 0`

New diagnostic findings:

- RefactoringMiner: `1` agent-evidence-only missed@10 file, role `test`, area
  `src/test/java/org/refactoringminer/mcp`.
- ctxhelm: `0` agent-evidence-only missed@10 files.
- ReAgent: `0` agent-evidence-only missed@10 files.
- VeriSchema: `10` agent-evidence-only missed@10 files, all role `test`.
- VeriSchema top agent-evidence-only areas:
  - `tests/agents=5`
  - `tests/evaluation=4`
  - `tests/core=1`

## Interpretation

The remaining difference between progressive next reads and the broader agent
evidence bundle is validation routing, not hidden source retrieval. The next
R&D should either:

- promote the right related-test evidence into progressive context-area reads,
- make agents explicitly consume `recommendedTests` alongside next-read paths,
- or prove that keeping those as validation evidence outside next reads is the
better product behavior.

Top-10 ranking changes should still be held to the Phase 200 proof bar: no
workflow/config regression, no top-10 churn without measured lift, and no
global runtime-threshold tuning.
