---
phase: 65
title: v2.5 Product Proof And Release Gate
date: 2026-05-30
status: research
requirements:
  - PROOF-01
  - PROOF-02
  - PROOF-03
  - PROOF-04
---

# Phase 65 Research: Product Proof And Release Gate

## Objective

Turn v2.5 retrieval evidence into a shippable or held product decision. The
proof must say when ctxpack beats lexical, when it merely matches, and when it
trails. Release gates should block unsafe or mixed default-promotion claims.

## Current Evidence

Phase 64 improved a specific RefactoringMiner gap family:

- RefactoringMiner Recall@10: `0.1375 -> 0.7392`
- selected wrapper-family misses: `7 -> 1`

But the default is not automatically production-ready:

- RefactoringMiner still trails lexical baseline in the final Phase 64 report:
  `0.7392` ctxpack vs `0.7792` lexical.
- ctxpack had a small Recall@10 regression: `0.2049 -> 0.1947`.
- Symbol protected evidence still has budget pressure under K=10.
- Test Recall@10 remains `0.0` on the two-repo proof.

## Required Product Behavior

The product proof should be honest enough to block premature release claims:

- `beat`: ctxpack clears lexical by more than the proof threshold.
- `match`: ctxpack is within threshold of lexical.
- `trail`: ctxpack falls below lexical by more than the threshold.
- `insufficient_evidence`: repo failed or produced no eval report.

Default promotion should require every required corpus to beat lexical while
remaining local-only and within runtime/protected-evidence constraints.

## Design Direction

Reuse `ctxpack eval proof` rather than inventing a new evaluator. Add release
decision fields to the proof JSON and Markdown:

- release decision: `promote`, `hold`, or `block`
- default-promotion boolean
- per-corpus verdicts with variant name, recall, lexical delta, test recall,
  protected miss rate, runtime, and notes

Wire `scripts/release-gate.sh` to fail optional benchmark proof when
`releaseGate.decision != "promote"`.

## Acceptance Criteria

- Product proof states beat/match/trail per corpus and variant.
- Optional release-gate benchmark proof blocks non-promote default retrieval.
- Docs explain current retrieval recommendation and proof boundary.
- Full workspace validation, CLI help, source-free E2E proof, and diff hygiene
  pass.
