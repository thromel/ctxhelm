# Phase 198 - Candidate Coverage Accounting

## Goal

Separate top-10 selected-file misses that were already generated as candidates
from misses with no candidate signal. This gives the next retrieval experiment a
hard diagnostic: fix ranking/selection pressure when misses are
candidate-recoverable, and fix candidate generation only when no-candidate
counts dominate.

## Rejected Experiments

### Source-Area Spillover Reserve

Added a same-area source spillover reserve after the dependency floor. Focused
tests passed and `/tmp/ctxhelm-rd/phase198-source-area-spillover-proof.json`
passed `scripts/check-product-proof.py`, but the four-repo metrics were
unchanged. The only changed VeriSchema commit replaced docs context with
source-area files while the target was `paper/pvldb/README.md`, so it did not
recover a missed retrieval target. The experiment was reverted.

### Docs Entrypoint Reserve

Added README/index-style docs sibling reserve. Focused tests passed and
`/tmp/ctxhelm-rd/phase198-docs-entrypoint-proof.json` passed the proof checker,
but it changed zero measured commits and did not improve recall. The experiment
was reverted.

### Local Metadata Reranker

Ran the existing eval-only local metadata reranker with
`/tmp/ctxhelm-rd/phase198-local-reranker-config.json`. The proof at
`/tmp/ctxhelm-rd/phase198-local-reranker-proof.json` blocked promotion. It
improved RefactoringMiner File Recall@10 from `0.8` to `1.0`, but regressed
ctxhelm, ReAgent, and VeriSchema file/source recall, so it is not promotable.

## Accepted Change

Historical eval reports now include:

```json
"candidateCoverageSummary": {
  "missedFileCountAt10": 39,
  "candidateRecoverableCount": 36,
  "noCandidateCount": 3,
  "sourceTextLogged": false
}
```

Each commit also records `candidateMissedFilesAt10`, a source-free path list
containing missed@10 retrieval targets that existed in the generated candidate
set. The product-proof checker validates the summary arithmetic and rejects any
candidate coverage summary that is not source-free.

## Fresh Proof

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase198-candidate-coverage-home \
  cargo run --release -p ctxhelm --locked -- eval proof \
  --config .planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json \
  --format json > /tmp/ctxhelm-rd/phase198-candidate-coverage-proof.json

python3 scripts/check-product-proof.py \
  /tmp/ctxhelm-rd/phase198-candidate-coverage-proof.json
```

Result: `releaseGate.decision = promote`.

Compared with Phase 197, retrieval and lexical-comparison metrics are unchanged:

- Average File Recall@10: `0.658268`
- Average lexical File Recall@10: `0.4795285`
- Average file delta: `+0.17873949`
- Average Agent Evidence Recall@10: `0.76052284`
- Average Context Recall@10: `0.7638889`
- All-file beat/match/trail: `3 / 0 / 1`, explained trail `1`, unexplained trail `0`
- Agent-evidence beat/match/trail: `3 / 1 / 0`
- Context beat/match/trail: `3 / 1 / 0`

Candidate coverage:

| Repository | Missed@10 | Candidate Recoverable | No Candidate |
| --- | ---: | ---: | ---: |
| ctxhelm | 12 | 11 | 1 |
| RefactoringMiner | 1 | 1 | 0 |
| ReAgent | 0 | 0 | 0 |
| VeriSchema | 39 | 36 | 3 |

## Interpretation

The dominant VeriSchema bottleneck is not broad missing candidate generation:
`36 / 39` missed@10 files were generated but ranked outside the selected
budget. Next experiments should target source-free selection pressure and
ranking allocation, using this candidate coverage field to reject changes that
only shuffle candidates without improving top-10 recall.
