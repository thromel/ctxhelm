# Phase 73 Broader Fixed-Corpus Fixture

Date: 2026-05-30

## Goal

Make the broader production-readiness probe reproducible from a committed source-free benchmark config instead of a temporary `/tmp` manifest.

## Artifact

Committed config:

```text
.planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json
```

The config covers:

- RefactoringMiner
- ctxhelm
- ReAgent
- VeriSchema

It uses `limit = 5`, `rankingBudget = 10`, local-only defaults, cloud transfer disabled, and pinned revisions for each repository. The ctxhelm corpus is pinned to the Phase 72 pre-test-seed history head so future ctxhelm commits do not silently change this probe. RefactoringMiner is pinned to a single recent commit range because a broader pinned range crosses very large historical snapshots that are intentionally bounded by ctxhelm's local eval runtime safeguards.

## Proof Command

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxhelm-phase73-pinned-broader-fixed-corpus-proof.json
```

## Result

The fixture is reproducible but still blocks broader promotion.

| Corpus | Status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected Miss-Rate@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | match | 1.0000 | 1.0000 | 1.0000 | 0.1000 |
| ctxhelm | beat | 0.3611 | 0.3056 | 0.0000 | 0.1633 |
| ReAgent | beat | 0.7143 | 0.5714 | 1.0000 | 0.2174 |
| VeriSchema | trail | 0.1507 | 0.0822 | 0.6614 | 0.1163 |

`releaseGate.decision = block`.

## Notes

- This fixture is optional evidence, not the required release gate.
- The required two-repo gate remains `.ctxhelm/e2e/v25-multirepo-baseline-config.json`.
- The blocked result is useful: it preserves the next production-readiness targets in a reproducible artifact rather than hiding them in an ad hoc local run.

## Remaining Gaps

- VeriSchema still misses the 0.80 validation-test floor on large multi-area commits.
- RefactoringMiner only matches lexical context recall in this pinned short-window probe.
- Protected evidence miss-rate remains non-zero across all broader corpora.
