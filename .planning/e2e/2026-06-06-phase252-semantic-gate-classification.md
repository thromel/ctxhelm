# Phase 252 - Semantic Gate Classification Fix

## Goal

Fix the misleading semantic R&D gate state where an eval-only local metadata
reranker regression could make the top-level semantic gate report `block`, even
though the semantic promotion candidate itself should be held as opt-in.

## Root Cause

`gate_decision_from_variants` received named regressions from both
`local_semantic` and `local_metadata_reranked`. The top-level decision is based
on `ctxhelm_default` versus `local_semantic`, but any named regression from any
variant forced `block`.

That conflated two different findings:

- semantic default promotion is not yet justified;
- the eval-only metadata reranker can still regress protected/default evidence.

The second finding should stay visible as a diagnostic, but it should not make
the semantic default-lift lane look blocked.

## Fix

- Top-level semantic promotion now blocks only on regressions from semantic
  promotion variants: `local_semantic`, `precision_enriched_semantic`, and
  `semantic_precision_full_hybrid`.
- Eval-only reranker regressions remain in `namedRegressions` and diagnostics.
- Added a focused unit test proving `local_metadata_reranked` regressions do not
  turn a semantic hold into a semantic block.

## Evidence

Source-free artifacts:

- `.ctxhelm/e2e/phase252-semantic-gate-refactoringminer.json`
- `.ctxhelm/e2e/phase252-semantic-gate-ctxhelm.json`
- `.ctxhelm/e2e/phase252-agent-run-claude-rd-breadth-suite.json`

Measured semantic gate results after the fix:

| Repo | Decision | Default Recall@K | Local Semantic Recall@K | Semantic Delta | Semantic-only target hits | Notes |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| RefactoringMiner | `hold` | `0.48541665` | `0.5104166` | `+0.025` | `1` | Small lift, not enough for default promotion. |
| ctxhelm | `hold` | `0.3212704` | `0.3212704` | `+0.000` | `1` | Neutral recall; keep semantic opt-in. |

Both reports preserve local-only privacy and record no source text, raw prompts,
raw transcripts, remote embeddings, or remote reranking.

The fresh Claude paired suite was attempted with Claude Code `2.1.163`, but the
client preflight observed rate limiting. The report is therefore correctly
`degraded`, and no retrieval-quality conclusion is drawn from that run. The
same report still confirms ctxhelm's source-free evidence generation path hit
all expected targets in assisted lanes before client consumption could be
measured.

## Decision

Semantic is no longer misclassified as blocked by an unrelated eval-only
reranker regression. The correct current state is:

- `local_fastembed` is available, local-only, and source-free;
- semantic remains opt-in because the measured lift is small or neutral;
- local metadata reranker remains eval/policy-gated because its regressions are
  still visible.

Next semantic R&D should target query/document construction or query-family
evaluation rather than default promotion.
