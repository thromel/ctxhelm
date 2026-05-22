---
phase: 61
title: Multi-Repo Quality Baselines
status: complete
completed: 2026-05-22
requirements_addressed:
  - BASE-01
  - BASE-02
  - BASE-03
  - BASE-04
---

# Phase 61 Summary: Multi-Repo Quality Baselines

## Completed

- Activated v2.5 Production Retrieval Quality.
- Archived v2.4 active requirements, roadmap, and phases under `.planning/milestones/`.
- Reused the existing source-free benchmark suite machinery instead of adding a parallel evaluator.
- Ran a real two-repo baseline over RefactoringMiner and ctxpack.
- Captured a concise source-free E2E summary in `.planning/e2e/2026-05-22-v25-multirepo-baseline.md`.

## Evidence

Command:

```bash
ctxpack eval benchmark --config .ctxpack/e2e/v25-multirepo-baseline-config.json --format json
```

Result:

- Repositories evaluated: 2/2
- Commits evaluated: 20
- Privacy: local-only, no remote embeddings, no remote reranking
- RefactoringMiner: default Recall@10 `0.7767`, lexical Recall@10 `0.7792`
- ctxpack: default Recall@10 `0.2270`, lexical Recall@10 `0.2742`

## Interpretation

Phase 61 confirms that ctxpack already has the right benchmark-suite foundation for multi-repo proof. The next quality work should not be another evaluator; it should improve retrieval quality where the proof shows gaps:

- ctxpack itself trails lexical by `-0.0472` Recall@10.
- ctxpack misses docs/scripts/planning families and compiler source files under tight budgets.
- RefactoringMiner is close to lexical parity but still exposes test-mapping and non-source gap families.

## Next

Phase 62 should run production local embeddings against this same two-repo baseline and only promote behavior if it beats lexical/default under source-free gates.
