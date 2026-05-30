---
phase: 63
title: Reranker And Fusion Promotion
status: complete
completed_at: 2026-05-30
requirements_addressed:
  - RANK-01
  - RANK-02
  - RANK-03
  - RANK-04
---

# Phase 63 Summary: Reranker And Fusion Promotion

## Outcome

Phase 63 is complete. ctxpack can now evaluate an explicit local metadata
reranker variant through existing eval/benchmark surfaces without adding MCP
tools or exposing source text in reports.

## Implemented

- Added `localMetadataReranker` to historical eval and benchmark suite configs.
- Added eval-only local metadata reranking over source-free candidate metadata.
- Added protected-evidence accounting for anchor, current-diff, lexical, and
  symbol signals.
- Added protected-evidence miss rates to historical eval, benchmark, and gate
  reports.
- Added gate-level named regressions when a variant demotes protected evidence
  kept by default.
- Added promotion holds for slow or token-inefficient variants.

## E2E Result

See `.planning/e2e/2026-05-30-phase63-reranker-fusion-promotion.md`.

The local metadata reranker:

- improved RefactoringMiner Recall@10 from `0.1375` to `0.6642`;
- regressed ctxpack Recall@10 from `0.2049` to `0.1927`;
- improved test recall on both repos;
- introduced protected-evidence miss-rate lift on RefactoringMiner;
- triggered `block` in the local gate because named regressions were detected.

## Decision

Do not promote the reranker as a default. Keep it source-free and measurable, but
move to Phase 64 gap-family retrieval fixes before attempting default promotion.

## Validation

```bash
cargo test -p ctxpack-compiler gate_decision -- --nocapture
cargo test -p ctxpack-compiler protected_evidence -- --nocapture
cargo test -p ctxpack historical_eval_report_renders_source_free_metrics -- --nocapture
cargo run -p ctxpack -- eval gate --limit 5 --budget 10 --format json
```
