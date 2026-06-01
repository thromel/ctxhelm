# Phase 75 Parent-Bounded History And Test Reserve

## Goal

Make historical eval snapshots closer to real local-agent conditions without
leaking source text or future commits, and make validation-test selection more
protective of tests that co-change with target files.

## Change

- Historical parent snapshots now get a source-free `.ctxhelm/eval-history.json`
  sidecar containing only commit SHAs and changed paths, bounded to the parent
  revision.
- Co-change retrieval falls back to that sidecar when a snapshot has no `.git`
  history.
- The sidecar is classified as generated so it cannot be selected as task
  context.
- Related-test selection gives co-changed validation tests a first-pass reserve
  before filling remaining test slots with generic test matches.

This keeps ctxhelm local-first and source-safe: the sidecar contains no source
text, no prompt text, and no future commits.

## Required Two-Repo Proof

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .ctxhelm/e2e/v25-multirepo-baseline-config.json \
  --format json > /tmp/ctxhelm-phase75-two-repo-proof.json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase75-two-repo-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase75-parent-history-test-reserve-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `beat` | 0.778 | 0.741 | 1.000 | 0.059 |
| ctxhelm | `beat` | 0.478 | 0.391 | 1.000 | 0.074 |

Gate decision: `promote`.

Compared with Phase 74, ctxhelm target protected miss-rate improved from 0.133
to 0.074 on the required proof. Context Recall@10 remains above lexical on both
required corpora.

## Broader Fixed-Corpus Probe

Command:

```bash
cargo run -p ctxhelm -- eval proof \
  --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json \
  --format json > /tmp/ctxhelm-phase75-broader-proof.json
```

Committed artifact:

- `.ctxhelm/e2e/phase75-broader-parent-history-test-reserve-proof.json`

Result:

| Corpus | Gate status | Context Recall@10 | Lexical Context Recall@10 | Test Recall@10 | Protected target miss@10 |
| --- | --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `match` | 1.000 | 1.000 | 1.000 | 0.000 |
| ctxhelm | `beat` | 0.361 | 0.306 | 0.000 | 0.100 |
| ReAgent | `beat` | 0.714 | 0.571 | 1.000 | 0.000 |
| VeriSchema | `trail` | 0.151 | 0.082 | 0.661 | 0.143 |

Gate decision: `block`.

The broader probe did not improve VeriSchema validation-test Recall@10. The two
remaining low-recall VeriSchema commits are title-only, multi-area changes where
bounded history and co-changed test reservation are still insufficient to infer
all changed tests within ten slots. Next work should target validation-test
diversification and task-title weakness detection rather than further
co-change plumbing.

## Validation

```bash
cargo test -p ctxhelm-index co_change_hints_use_eval_history_sidecar_when_snapshot_has_no_git -- --nocapture
cargo test -p ctxhelm-index policy_classifies_common_credentials_and_generated_families -- --nocapture
cargo test -p ctxhelm-compiler selection_promotes_cochanged_validation_tests -- --nocapture
cargo run -p ctxhelm -- eval proof --config .ctxhelm/e2e/v25-multirepo-baseline-config.json --format json
python3 scripts/check-product-proof.py /tmp/ctxhelm-phase75-two-repo-proof.json
cargo run -p ctxhelm -- eval proof --config .planning/e2e/2026-05-30-phase73-broader-fixed-corpus-config.json --format json
```

