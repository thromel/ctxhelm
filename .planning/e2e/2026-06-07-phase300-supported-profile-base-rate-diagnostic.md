# Phase 300 - Supported Profile Base-Rate Diagnostic

## Scope

Phase 299 showed the source-free supported-shape tail-slot variant is safe but
too sparse for promotion: one lifted VeriSchema older commit, zero regressions,
and zero default-only target churn across eight stable slices. Phase 300 makes
the broader supported semantic candidate base rate machine-readable instead of
adding another overfit eval variant.

This phase adds `supportedSemanticCandidateProfileSummary` to historical eval
reports and semantic gate reports. The summary is source-free and read-only. It
counts generated `supportedSemanticCandidateProfilesAt10` rows by:

- query family
- file role
- path family
- support family

Each shape reports profile count, target count, non-target count, target
precision, commit count, target-commit count, `thinCell`, and
`repeatedTargetSupport`. Thin cells are profile counts below three; repeated
target support requires at least two target commits for the same shape inside
the report.

Claude Code Opus 4.8 reviewed the Phase 299 profile surface and recommended a
read-only base-rate diagnostic instead of another predictor variant. The
observed precision and thin-cell counts below match that recommendation.

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
  > .ctxhelm/e2e/phase300-supported-profile-summary-gate-verischema-older.json
```

## Result

Across RefactoringMiner, ctxhelm, ReAgent, and VeriSchema older/recent ranges:

| Metric | Value |
| --- | ---: |
| supported profiles | `787` |
| target profiles | `22` |
| non-target profiles | `765` |
| global target precision | `0.027954` |
| per-slice shape rows | `171` |
| per-slice positive shape rows | `18` |
| per-slice thin cells | `90` |
| per-slice repeated-target shape rows | `1` |

Per-slice profile precision:

| Slice | Profiles | Targets | Non-targets | Precision | Repeated-target shapes |
| --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner older | 34 | 0 | 34 | `0.000000` | 0 |
| RefactoringMiner recent | 38 | 1 | 37 | `0.026316` | 0 |
| ctxhelm older | 142 | 7 | 135 | `0.049296` | 0 |
| ctxhelm recent | 131 | 9 | 122 | `0.068702` | 1 |
| ReAgent older | 121 | 2 | 119 | `0.016529` | 0 |
| ReAgent recent | 162 | 0 | 162 | `0.000000` | 0 |
| VeriSchema older | 80 | 1 | 79 | `0.012500` | 0 |
| VeriSchema recent | 79 | 2 | 77 | `0.025316` | 0 |

The original VeriSchema older lift is now visible as a thin cell:

```text
symbol_identifier source python_source dependency_co_change
profileCount=1 targetCount=1 thinCell=true repeatedTargetSupport=false
exampleTargetPaths=schema_agent/core/state.py
```

The broader base rate argues against adding another supported-profile reranker
variant: the trigger surface is mostly non-targets, and the clean-looking cells
are generally singletons.

## Decision

Keep supported semantic candidate shaping diagnostic-only. The useful artifact
from this branch is the durable base-rate summary, not another eval-only
predictor. Future semantic R&D should first find repeated, source-free
candidate-retention support or evaluate a held-out learned separator. Do not
promote Jina, tail-slot semantic insertion, or a supported-profile shape as
runtime/default policy from this evidence.
