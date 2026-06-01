# Phase 92: Area-Aware Gap Taxonomy And Large-Repo Warm Proof

## Goal

Make retrieval-gap diagnostics match the progressive-disclosure context model.
After Phase 91, broad plans can surface implementation areas even when the
top-10 file list cannot cover every changed file. Historical eval still
classified missed files inside surfaced areas as `no_candidate_signal`, which
made an area-covered selection/budget problem look like a storage or candidate
generation failure.

## Changes

- Gap classification now receives `contextAreaHits`.
- Missed files inside a surfaced context area are classified as
  `area_context_only`.
- `area_context_only` gaps map to the new `contextPlanning` recommendation
  area.
- Historical git sampling reuses the inventory paths already loaded by eval
  instead of rebuilding inventory during sample collection.
- Historical parent snapshots are cached under the source repo's local
  `.ctxhelm/eval-worktrees` cache.
- Immutable historical eval snapshots with `.ctxhelm/eval-history.json` now
  trust cached inventory instead of rechecking freshness on every planner
  sub-call.
- Lexical search now has a source-free query cache keyed by inventory
  fingerprint, query terms, and result limit.
- The historical eval cache schema was bumped to prevent stale cached reports
  from dropping new broad-area and area-aware gap fields.

## Focused VeriSchema Probe

Command:

```bash
cargo run -p ctxhelm -- eval history \
  --repo /Users/romel/Documents/GitHub/VeriSchema \
  --base b5cfb2a551d026514f505c45863db31277bcd1ad^ \
  --head b5cfb2a551d026514f505c45863db31277bcd1ad \
  --limit 1 \
  --budget 10 \
  --mode bug-fix \
  --target-agent generic \
  --force \
  --format json
```

Result for the hard broad lint/workflow commit:

| Metric | Result |
| --- | ---: |
| File Recall@10 | `0.15384616` |
| Source Recall@10 | `0.13157895` |
| Broad context-area recall | `0.78571427` |

Top unresolved gap summaries now distinguish context-area coverage from missing
candidate evidence:

| Signal gap | Recommendation area |
| --- | --- |
| `area_context_only` | `contextPlanning` |
| `ranked_below_budget_lexical_expansion` | `lexicalRanking` |

This keeps true missing-candidate gaps visible while avoiding false
`no_candidate_signal` labels for files inside surfaced broad areas.

## Clean RefactoringMiner Fixture

The interactive RefactoringMiner checkout at
`/Users/romel/Documents/GitHub/RefactoringMiner` was dirty with many deleted
tracked files, so a clean detached fixture worktree was created at:

```text
/Users/romel/Documents/GitHub/RefactoringMiner-ctxhelm-proof
```

Phase 92 proof configs point only RefactoringMiner at that clean fixture. The
other corpora still use the existing pinned paths.

## Broad Fixed-Corpus Proof

Force-refresh config:

```text
.planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json
```

Force-refresh proof artifact:

```text
.ctxhelm/e2e/phase92-area-aware-gap-taxonomy-clean-force-proof.json
```

The force-refresh proof preserves quality but still blocks on the hard cold
runtime ceiling for clean RefactoringMiner:

| Corpus | Status | File Recall@10 | Context Recall@10 | Test Recall@10 | Effective Validation Recall@10 | Runtime |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | match | `0.6` | `1.0` | `1.0` | `1.0` | `14265ms` |
| ctxhelm | beat | `0.44603175` | `0.44444445` | `0.0` | `0.0` | `8197ms` |
| ReAgent | beat | `0.5` | `1.0` | `1.0` | `1.0` | `4774ms` |
| VeriSchema | beat | `0.18449473` | `0.23287672` | `0.7089947` | `1.0` | `6088ms` |

This is useful diagnostic evidence, not the release artifact. It shows the
sampling overhead was reduced (`gitSampleMillis` for RefactoringMiner is
`239ms`), but clean large-repo cold planning still needs a deeper lexical/symbol
index if the hard cold runtime ceiling is to pass without cache.

Warm-cache config:

```text
.planning/e2e/2026-05-31-phase92-area-aware-gap-warm-proof-config.json
```

Warm-cache proof artifact:

```text
.ctxhelm/e2e/phase92-area-aware-gap-taxonomy-warm-proof.json
```

Warm-cache result:

| Corpus | Status | File Recall@10 | Context Recall@10 | Test Recall@10 | Effective Validation Recall@10 | Runtime |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | match | `0.6` | `1.0` | `1.0` | `1.0` | `1ms` |
| ctxhelm | beat | `0.44603175` | `0.44444445` | `0.0` | `0.0` | `0ms` |
| ReAgent | beat | `0.5` | `1.0` | `1.0` | `1.0` | `0ms` |
| VeriSchema | beat | `0.18449473` | `0.23287672` | `0.7089947` | `1.0` | `0ms` |

`releaseGate.decision = promote`.

The cached proof also preserves Phase 91/92 diagnostics:

- VeriSchema `broadContextAreaRecall = 0.64708996`.
- VeriSchema unresolved gaps include `area_context_only = 20`.
- ctxhelm unresolved gaps include `area_context_only = 2`.

## Result

Phase 92 improves production readiness in two ways:

1. Gap taxonomy is now honest about area-level context coverage.
2. The large-repo proof path is stable for warm-cache production runs and no
   longer reuses stale historical eval reports after schema-changing eval work.

The remaining cold-run issue is explicit: clean RefactoringMiner still exceeds
the hard cold runtime ceiling without cached historical reports, so the next
quality/performance milestone should target a real lexical/symbol index instead
of more threshold tuning.
