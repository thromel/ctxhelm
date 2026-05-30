# Phase 71 E2E: Archive Artifact Dampening

Date: 2026-05-30

## Purpose

Reduce docs/scripts storage-gap noise from historical planning archives without
removing those artifacts from search. The Phase 69 proof promoted default local
retrieval, but the ctxpack corpus still showed protected evidence pressure and
old `.planning/e2e/*.json` plus `.planning/milestones/**` artifacts competing
with active source and current planning files.

## Change

- Lexical search now dampens archive context artifacts:
  - `.planning/milestones/**`
  - `.planning/e2e/**/*.json`
- Archive artifacts remain searchable and can still appear when relevant.
- The fixed-budget selector reserves a small symbol slot only when archive
  lexical artifacts are present, so ordinary repositories are not affected.

## Proof Command

```bash
cargo run -p ctxpack -- eval proof \
  --config /tmp/ctxpack-ab-config.json \
  --format json > /tmp/ctxpack-phase71-final-proof.json

cp /tmp/ctxpack-phase71-final-proof.json \
  .ctxpack/e2e/phase71-archive-artifact-dampening-proof.json

python3 scripts/check-product-proof.py \
  .ctxpack/e2e/phase71-archive-artifact-dampening-proof.json
```

## Current-History A/B Result

Compared against commit `841dd38` on the same current-history suite:

| Corpus | Metric | Before | After |
| --- | ---: | ---: | ---: |
| RefactoringMiner | Context Recall@10 | 0.7778 | 0.7778 |
| RefactoringMiner | File Recall@10 | 0.7392 | 0.7392 |
| RefactoringMiner | Protected miss-rate@10 | 0.0526 | 0.0526 |
| ctxpack | Context Recall@10 | 0.3117 | 0.5195 |
| ctxpack | File Recall@10 | 0.2909 | 0.4986 |
| ctxpack | Protected miss-rate@10 | 0.2500 | 0.1633 |

Release gate result: `promote`.

## Interpretation

- The storage-gap fix is targeted: RefactoringMiner is unchanged because it does
  not have ctxpack planning archive artifacts.
- ctxpack improves substantially on context recall and file recall.
- Protected evidence miss-rate improves on ctxpack from 25.00% to 16.33%.
- Test Recall@10 remains 1.0 on both corpora.
- This does not complete all production-readiness work. Parser/precision misses
  and broader multi-repo repeated-lift validation remain.
