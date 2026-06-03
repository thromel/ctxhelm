# Phase 199 - Candidate Miss Pressure Profiles

## Goal

Use the Phase 198 candidate coverage split to expose where generated-but-
unselected misses concentrate. The next ranking experiment should know whether
selection pressure comes from source files, tests, docs, particular context
areas, or particular signal families.

## Rejected Experiment

Before adding diagnostics, a bounded broad source-area floor was tested against
the four-repo proof:

- Proof: `/tmp/ctxhelm-rd/phase199-broad-area-floor-proof.json`
- Result: `releaseGate.decision = promote`, but every retrieval and lexical-
  comparison metric was unchanged.
- One VeriSchema commit changed selected context by replacing docs with source
  files, but it recovered no target file.

The selector was reverted. The issue needs better pressure visibility before
another ranking change.

## Accepted Change

`candidateCoverageSummary` now includes:

```json
{
  "candidateRecoverableRoleCounts": {"source": 16, "test": 17},
  "candidateRecoverableSignalCounts": {
    "co_change": 17,
    "related_test": 17,
    "dependency": 12
  },
  "noCandidateRoleCounts": {"source": 3},
  "topCandidateRecoverableAreas": [
    {"contextArea": "schema_agent/agents", "missedCount": 7}
  ]
}
```

Each commit also emits `candidateMissedFileProfilesAt10` with the missed path,
role, context area, and source-free signal names.

## Fresh Proof

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase199-candidate-pressure-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase199-candidate-pressure-proof.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase199-candidate-pressure-proof.json
```

Result: `releaseGate.decision = promote`.

Phase 198 retrieval and lexical-comparison metrics are unchanged:

- Average File Recall@10: `0.658268`
- Average lexical File Recall@10: `0.4795285`
- Average file delta: `+0.17873949`
- Average Agent Evidence Recall@10: `0.76052284`
- Average Context Recall@10: `0.7638889`
- All-file beat/match/trail: `3 / 0 / 1`, explained trail `1`, unexplained trail `0`
- Agent-evidence beat/match/trail: `3 / 1 / 0`
- Context beat/match/trail: `3 / 1 / 0`

## Pressure Findings

VeriSchema:

- Candidate recoverable: `36 / 39`
- Recoverable roles: `source=16`, `test=17`, `docs=1`, `unknown=2`
- Recoverable signals: `co_change=17`, `related_test=17`, `dependency=12`, `lexical_expansion=9`, `lexical=5`
- No-candidate roles: `source=3`
- Top recoverable areas: `schema_agent/agents=7`, `tests/agents=6`, `tests/evaluation=6`, `schema_agent/core=3`, `schema_agent/evaluation=3`

ctxhelm:

- Candidate recoverable: `11 / 12`
- Recoverable roles: `source=6`, `docs=5`
- Recoverable signals: `co_change=10`, `docs=5`, `lexical=3`, `lexical_expansion=3`
- No-candidate roles: `source=1`
- Top recoverable areas: `crates/ctxpack-compiler=3`, `crates/ctxpack-index=3`, `docs=3`

## Interpretation

The next selection experiment should target broad generated-but-unselected
pressure in `schema_agent/agents`, `tests/agents`, and `tests/evaluation`, with
careful protection against replacing docs/config evidence when the target is
docs-only. A generic area-diversity floor is too blunt; pressure-aware ranking
needs to consider role, signal, and context-area saturation together.
