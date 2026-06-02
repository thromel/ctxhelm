# Phase 180: Graph Edge Ablations

## Goal

Move GraphRAG evidence from candidate-pressure diagnostics toward measured
edge-family lift. Phase 179 showed which graph edge labels produced candidates,
selected paths, target hits, and target misses, but it still could not answer
whether an edge family was contributing unique top-10 retrieval value.

## Change

Historical eval reports now include source-free `graphEdgeAblations`.

Each row records:

- `edgeLabel`
- `evaluatedCommits`
- `affectedCommitCount`
- `removedSelectedAt10Count`
- `removedTargetHitAt10Count`
- `metrics`
- `recallDeltaAtK`
- `recallLiftVsLexicalAtK`

The ablation is conservative. Disabling an edge label removes a selected file
only when that file's only evidence is the disabled dependency edge family.
Files with lexical, symbol, test, history, co-change, or another graph edge
label stay in the ranking. This avoids overstating graph impact for files that
have independent support.

Markdown `eval history` output now renders a `Graph Edge Ablations` section.

## Proof

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase180-graph-ablation-home \
  cargo run -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json \
  --format json > /tmp/ctxhelm-rd/phase180-graph-edge-ablation-proof.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase180-graph-edge-ablation-proof.json
```

Result:

- `releaseGate.decision = promote`
- `allFileClaim = mixed`
- `allFileBeatCount = 3`
- `allFileMatchCount = 0`
- `allFileTrailCount = 1`
- `allFileExplainedTrailCount = 1`
- `allFileUnexplainedTrailCount = 0`
- Average File Recall@10: ctxhelm `0.61190045` vs lexical `0.45709258`
- Average lift at 10: `+0.15480788`

Graph edge ablation evidence:

| Corpus | Edge label | Affected commits | Removed selected@10 | Removed target hits@10 | Recall delta@K | Lift vs lexical@K |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | imports | 1 | 1 | 0 | `0.0` | `-0.19999999` |
| ctxhelm | imports | 1 | 1 | 0 | `0.0` | `+0.36031747` |
| ReAgent | imports | 1 | 1 | 0 | `0.0` | `+0.4` |
| VeriSchema | imports | 2 | 2 | 1 | `-0.00512819` | `+0.05378583` |
| VeriSchema | python_reexport | 0 | 0 | 0 | `0.0` | `+0.05891402` |

## Interpretation

The new ablation shows that most selected import-only graph paths are currently
non-target context in the fixed proof, but VeriSchema does have one exclusive
import-backed target hit. Python re-export edges add candidate/selection pressure
without exclusive top-10 target lift on this fixture.

The next GraphRAG R&D target should be edge-family budget allocation: imports
are useful enough to keep, but broad VeriSchema still has many import-backed
target misses, so the issue is selecting the right import-neighborhood targets
inside a tight top-10 budget rather than adding more raw graph edges.
