# Phase 63 E2E: Reranker And Fusion Promotion

Date: 2026-05-30

## Objective

Evaluate source-free reranker/fusion promotion gates and decide whether the
local metadata reranker should be promoted as a stronger default.

## Commands

```bash
cargo test -p ctxpack-compiler gate_decision -- --nocapture
cargo test -p ctxpack-compiler protected_evidence -- --nocapture
cargo test -p ctxpack historical_eval_report_renders_source_free_metrics -- --nocapture

cargo run -p ctxpack -- \
  eval benchmark --config .ctxpack/e2e/phase62-default-config.json --format json

cargo run -p ctxpack -- \
  eval benchmark --config .ctxpack/e2e/phase63-local-reranker-config.json --format json

cargo run -p ctxpack -- \
  eval gate --limit 5 --budget 10 --format json
```

Large JSON reports were kept under ignored `.ctxpack/e2e/`.

## Variant Results

| Repo | Default Recall@10 | Reranked Recall@10 | Delta | Default MRR@K | Reranked MRR@K | Test Recall@10 delta | Protected miss-rate delta | Runtime delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 0.1375 | 0.6642 | +0.5267 | 0.1500 | 0.6125 | +1.0000 | +0.1509 | +13.4s |
| ctxpack | 0.2049 | 0.1927 | -0.0122 | 0.6333 | 0.7167 | +0.5000 | +0.0000 | -0.6s |

## Gate Result

`ctxpack eval gate --limit 5 --budget 10 --format json` returned:

- decision: `block`
- reason: `Blocked because named regressions were detected.`
- default Recall@K: `0.1870`
- local metadata reranked Recall@K: `0.1688`
- local metadata reranked precision@K: `0.2200`
- local metadata reranked token efficiency: `0.4583`
- local metadata reranked protected miss rate@10: `0.2063`
- source text logged: `false`
- privacy: local-only

Named regressions included:

- `9d6baddb0ce7`: local metadata reranker retrieved fewer gold changed files than default.
- `f247e826e4ac`: local metadata reranker demoted `crates/ctxpack-core/src/contracts.rs`, a protected evidence path kept by default.
- `feeaac955897`: local metadata reranker demoted protected script evidence kept by default.

## Decision

Hold/block promotion. The local metadata reranker is source-free and can produce
large gains on RefactoringMiner, but current evidence is mixed and the gate found
named protected-evidence regressions. It remains an eval/policy-gated variant,
not a default.

## Follow-up

Phase 64 should target measured gap families instead of promoting the broad
reranker:

- RefactoringMiner: `no_candidate_signal` under `src/main/java/org/refactoringminer/astDiff/matchers/wrappers/*.java`.
- ctxpack: docs/planning `no_candidate_signal` and compiler `ranked_below_budget_dependency`.
