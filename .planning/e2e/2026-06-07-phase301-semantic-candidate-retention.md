# Phase 301 - Semantic Candidate Retention

## Scope

Phase 300 showed supported semantic candidate shapes have a low target base
rate, so adding another hand-written shape predictor would overfit. Phase 301
adds a source-free retention diagnostic instead: it measures supported semantic
candidates that are retained in the current top K versus supported semantic
candidates generated but dropped.

This phase adds `semanticCandidateRetentionSummary` to historical eval and
semantic gate reports. It is read-only and does not change ranking. Each family
uses the same source-free shape dimensions as the supported profile summary:

- query family
- file role
- path family
- support family

Each family reports retained/dropped target counts, retained/dropped non-target
counts, target retention rate, non-target drop rate, `thinCell`, and bounded
example paths. Claude Code Opus 4.8 recommended this grouped 2x2 retention view
over another predictor because it measures whether the current top-K boundary
separates targets from non-targets.

## Proof Command

The proof artifacts were regenerated after a clean feature-enabled build:

```bash
cargo build -p ctxhelm --features local-embeddings --locked
```

Each history report used:

```bash
./target/debug/ctxhelm eval history \
  --repo <fixture> \
  --base <range-base> --head <range-head> \
  --limit 20 --budget 10 \
  --semantic \
  --semantic-provider local_fastembed \
  --semantic-model JinaEmbeddingsV2BaseCode \
  --semantic-query-mode candidate-path-hints \
  --format json
```

The semantic gate path was checked with:

```bash
./target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --base 6dd8f127cdbd --head 85920ebe534c \
  --limit 20 --budget 10 \
  --semantic-provider local_fastembed \
  --semantic-model JinaEmbeddingsV2BaseCode \
  --semantic-query-mode candidate-path-hints \
  --format json \
  > .ctxhelm/e2e/phase301-semantic-candidate-retention-gate-verischema-older.json
```

## Result

Across RefactoringMiner, ctxhelm, ReAgent, and VeriSchema older/recent ranges:

| Metric | Value |
| --- | ---: |
| supported semantic retention profiles | `1071` |
| retained profiles | `284` |
| dropped profiles | `787` |
| retained targets | `52` |
| dropped targets | `22` |
| retained non-targets | `232` |
| dropped non-targets | `765` |
| target retention rate | `0.702703` |
| non-target drop rate | `0.767302` |
| recoverable dropped-target family rows | `12` |

Per-slice retention:

| Slice | Profiles | Retained targets | Dropped targets | Retained non-targets | Dropped non-targets | Target retention | Non-target drop | Recoverable rows |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner older | 44 | 0 | 0 | 10 | 34 | `0.000000` | `0.772727` | 0 |
| RefactoringMiner recent | 59 | 3 | 1 | 18 | 37 | `0.750000` | `0.672727` | 1 |
| ctxhelm older | 171 | 8 | 7 | 21 | 135 | `0.533333` | `0.865385` | 4 |
| ctxhelm recent | 168 | 19 | 9 | 18 | 122 | `0.678571` | `0.871429` | 5 |
| ReAgent older | 145 | 2 | 2 | 22 | 119 | `0.500000` | `0.843972` | 1 |
| ReAgent recent | 181 | 1 | 0 | 18 | 162 | `1.000000` | `0.900000` | 0 |
| VeriSchema older | 165 | 12 | 1 | 73 | 79 | `0.923077` | `0.519737` | 0 |
| VeriSchema recent | 138 | 7 | 2 | 52 | 77 | `0.777778` | `0.596899` | 1 |

The targeted VeriSchema older lift remains visible as a thin dropped-target
cell:

```text
symbol_identifier source python_source dependency_co_change
retainedTargetCount=0 droppedTargetCount=1 retainedNonTargetCount=0 droppedNonTargetCount=0
exampleDroppedTargetPaths=schema_agent/core/state.py
```

The broader retention view is more useful than the Phase 300 base-rate alone:
there are 22 dropped supported targets, but the same surface also drops 765
non-targets and keeps 232 non-targets. That means runtime promotion still needs
a held-out separator or stronger candidate-retention rule; the current evidence
does not justify a default policy.

## Decision

Keep `semanticCandidateRetentionSummary` diagnostic-only. Use it as the next
held-out learned-separator input because it exposes retained/dropped target and
non-target cells without changing ranking. Do not promote supported semantic
candidate retention as runtime/default policy from this observational proof.
