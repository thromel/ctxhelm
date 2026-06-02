# Phase 179: Graph Edge Profiles

## Goal

Make GraphRAG-style retrieval evidence more measurable without changing
ranking. The current proof already exposes signal-level ablations, but the graph
channel was still collapsed into one `Dependency` signal. That made it hard to
see whether imports, Python re-exports, or precision edges were helping or just
adding candidate pressure.

## Change

Historical eval reports now include source-free `graphEdgeProfiles` at report
and commit level. Each row records:

- `edgeLabel`
- `candidateCount`
- `selectedAt10Count`
- `retrievalTargetCount`
- `retrievalTargetHitAt10Count`
- `retrievalTargetMissedAt10Count`

Markdown `eval history` output also renders a `Graph Edge Profiles` section.

This is diagnostic only. It does not alter candidate generation, ranking,
context packs, MCP tools, or release-gate thresholds.

## Proof

Cold command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase179-graph-profile-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json \
  --format json > /tmp/ctxhelm-rd/phase179-graph-edge-profiles-proof.json
```

The cold proof preserved recall and lexical comparison metrics but blocked on
runtime only:

- `releaseGate.decision = block`
- Reason: RefactoringMiner exceeded the `5000ms` per-commit runtime ceiling.
- RefactoringMiner runtime: `11709ms`
- Recall metrics matched the warm proof shape.

Warm command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase179-graph-profile-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json \
  --format json > /tmp/ctxhelm-rd/phase179-graph-edge-profiles-warm-proof.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase179-graph-edge-profiles-warm-proof.json
```

Warm result:

- `releaseGate.decision = promote`
- `allFileClaim = mixed`
- `allFileBeatCount = 3`
- `allFileMatchCount = 0`
- `allFileTrailCount = 1`
- `allFileExplainedTrailCount = 1`
- `allFileUnexplainedTrailCount = 0`
- Average File Recall@10: ctxhelm `0.61190045` vs lexical `0.45709258`
- Average file delta: `+0.15480787`
- Average agent-evidence delta: `+0.2570628`
- Average context delta: `+0.30652046`

Graph edge profile evidence:

| Corpus | Edge label | Candidates | Selected@10 | Targets | Target hits@10 | Target misses@10 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | imports | 23 | 1 | 0 | 0 | 0 |
| ctxhelm | imports | 55 | 8 | 4 | 3 | 1 |
| ReAgent | imports | 78 | 4 | 0 | 0 | 0 |
| VeriSchema | imports | 101 | 19 | 28 | 6 | 22 |
| VeriSchema | python_reexport | 8 | 1 | 0 | 0 | 0 |

## Interpretation

The new diagnostic proves graph evidence is present, source-free, and measurable
by edge family. It also points to a concrete next R&D target: broad VeriSchema
source misses are not caused by absent import edges alone. `imports` produces
many target candidates but only six target hits at K=10, while Python re-export
edges currently add candidate pressure without target hits on this fixture.

Future graph work should therefore focus on edge-family ranking and budget
allocation, not just adding more graph edges.
