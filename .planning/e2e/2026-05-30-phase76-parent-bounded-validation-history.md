# Phase 76 Parent-Bounded Validation History

## Goal

Use parent-bounded source-free history in historical eval where it is reliable:
validation-test discovery. Avoid using partial snapshot history as a general
source-ranking signal because historical eval snapshots intentionally extract
only candidate/label paths, not the full repository.

## Change

- Added a `HistoryMode::ValidationOnly` planner path for historical eval
  snapshots.
- Historical parent snapshots now use `.ctxhelm/eval-history.json` to enrich
  related-test discovery and command generation, but do not use that partial
  snapshot history to rank non-test target files.
- Co-changed tests that come from history are enriched from the safe test map so
  they carry runnable validation commands.
- Bumped the historical eval cache schema to avoid reusing stale source-free
  reports generated before the history-mode split.

This keeps the proof source-safe: the sidecar contains commit SHAs and changed
paths only, no source snippets, prompt text, terminal logs, or future commits.

## Required Two-Repo Proof

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .ctxhelm/e2e/v25-multirepo-baseline-config.json \
  --format json > /tmp/ctxhelm-phase76-two-repo-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase76-two-repo-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase76-parent-bounded-validation-history-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `beat` | 0.778 | 0.741 | 1.000 | 0.059 |
| ctxhelm | `beat` | 0.444 | 0.361 | 1.000 | 0.040 |

Gate decision: `promote`.

## Broader Fixed-Corpus Probe

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxhelm-phase76-broader-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase76-broader-parent-bounded-validation-history-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `match` | 1.000 | 1.000 | 1.000 | 0.000 |
| ctxhelm | `beat` | 0.361 | 0.306 | 0.000 | 0.100 |
| ReAgent | `beat` | 0.714 | 0.571 | 1.000 | 0.000 |
| VeriSchema | `trail` | 0.151 | 0.082 | 0.709 | 0.143 |

Gate decision: `block`.

Compared with Phase 75, VeriSchema validation-test Recall@10 improved from
`0.661` to `0.709`, but remains below the broader 0.80 floor. The two
remaining low-recall commits are still broad, multi-area validation changes
where ten test slots are insufficient to cover all changed tests.

## Validation

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler historical_eval_uses_parent_bounded_sidecar_for_cochanged_tests -- --nocapture
cargo test -p ctxhelm-compiler historical_eval_uses_parent_snapshot_without_future_context -- --nocapture
cargo test -p ctxhelm-compiler selection_promotes_cochanged_validation_tests -- --nocapture
cargo run -p ctxhelm -- eval proof --config .ctxhelm/e2e/v25-multirepo-baseline-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase76-two-repo-proof.json
cargo run -p ctxhelm -- eval proof --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json --format json
```
