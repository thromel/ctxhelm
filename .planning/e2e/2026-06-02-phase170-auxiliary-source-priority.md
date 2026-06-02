# Phase 170: Auxiliary Source Priority

## Goal

Improve broad source ranking without expanding context budgets by preventing auxiliary example/demo files from consuming source-floor slots ahead of implementation files.

This phase followed the Phase 169 clean proof and used the same four-repo clean-cold fixture config:

` .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json`

## Baseline

After restoring the renamed fixture directory, the clean-cold proof promoted across all four corpora:

- Decision: `promote`
- Evaluated repositories: `4`
- Evaluated commits: `16`
- Average File Recall@10: `0.54832906`
- Average lexical baseline Recall@10: `0.45709258`
- Average ctxhelm lift@10: `+0.09123646`
- Average Test Recall@10: `0.67989415`
- Average brief token ROI: `1.3125`
- VeriSchema Source Recall@10: `0.2624269`

Remaining measured pressure:

- VeriSchema still had source gaps in broad Python implementation areas.
- High-scoring auxiliary source paths such as `examples/full_workflow_example.py` could enter constrained top-10 context before implementation paths.

## Change

Source lexical and symbol floors now rank implementation roots before auxiliary roots such as:

- `examples`
- `example`
- `demo`
- `demos`
- `sample`
- `samples`
- `notebook`
- `notebooks`
- benchmark-style roots

The change is source-free: it uses only path metadata and existing candidate role labels.

## Rejected Variant

The first variant also demoted `scripts/`.

That improved VeriSchema slightly but regressed ReAgent because `scripts/run_second_family_end_to_end.py` is product source in that corpus:

- ReAgent File Recall@10 dropped from `0.8` to `0.5`.
- ReAgent Source Recall@10 dropped from `1.0` to `0.0`.

`scripts` was therefore removed from the auxiliary-root list and remains a normal source root.

## Final Proof

Final proof artifact:

`/tmp/ctxhelm-rd/phase170-aux-source-priority-noscripts-proof.json`

Final clean-cold four-repo proof:

- Decision: `promote`
- Average File Recall@10: `0.5499957`
- Average lexical baseline Recall@10: `0.45709258`
- Average ctxhelm lift@10: `+0.09290312`
- Average Test Recall@10: `0.67989415`
- Average brief token ROI: `1.375`

Per-corpus movement:

| Corpus | File Recall@10 before | File Recall@10 after | Context Recall@10 before | Context Recall@10 after |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | `0.8` | `0.8` | `1.0` | `1.0` |
| ctxhelm | `0.41904765` | `0.41904765` | `0.41666666` | `0.41666666` |
| ReAgent | `0.8` | `0.8` | `1.0` | `1.0` |
| VeriSchema | `0.17426848` | `0.18093514` | `0.21917808` | `0.23287672` |

VeriSchema source-channel movement:

- Source Recall@10 before: `0.2624269`
- Source Recall@10 after: `0.2763158`
- Test Recall@10 unchanged: `0.71957666`
- Effective validation recall unchanged: `1.0`

## Validation

Completed during the phase:

- `cargo fmt --check`
- `cargo test -p ctxhelm-compiler selection_prefers_implementation_sources_over_auxiliary_examples_in_source_floor --locked`
- `cargo run -p ctxhelm --locked -- eval proof --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json --format json`

Full workspace validation is run as closeout.
