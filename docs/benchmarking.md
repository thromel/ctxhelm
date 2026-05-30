# Retrieval Benchmarking

ctxpack v1.2 uses source-free benchmark suites to answer the product question:

> Does ctxpack help an agent choose better files and tests than native search alone?

The benchmark runner reuses `ctxpack eval history` for each configured repository. It records file/test recall, lexical and no-context baseline comparison, signal ablations, token ROI, grouped retrieval gaps, skipped path counts, runtime diagnostics, and privacy status without storing source snippets, prompt text, or commit subjects.

v2.3 treats benchmark suites as fixed corpus manifests. Older suite files still work, but v2.3+ manifests should include a manifest version, corpus ID, privacy label, revision range ID, and optional locked baseline metadata so quality claims are reproducible.

Phase 58 adds source-free query construction traces to prepare-task and historical eval commit rows. These traces record extracted paths, stack frames, symbols, error terms, domain terms, commit clues, retriever query sets, and fusion controls. They intentionally store task hashes and bounded facets instead of raw prompts or source snippets.

## Suite File

Benchmark suites are JSON files. Paths may be absolute or relative to the suite file.

```json
{
  "manifestVersion": "ctxpack-benchmark-corpus-v2.3",
  "name": "retrieval-quality-smoke",
  "corpusId": "ctxpack-local-retrieval-quality-smoke",
  "privacyLabel": "source_free_local",
  "description": "Bounded source-free retrieval benchmark over local repos",
  "defaults": {
    "limit": 20,
    "rankingBudget": 10,
    "mode": "bug_fix",
    "targetAgent": "codex",
    "semanticEnabled": false,
    "semanticProvider": "local_hash",
    "semanticModel": "ctxpack-local-hash-v1",
    "semanticDimensions": 64,
    "cacheEnabled": true,
    "forceRefresh": false,
    "parallelism": 4,
    "roleFilters": ["source", "test"]
  },
  "repositories": [
    {
      "name": "ctxpack",
      "path": ".",
      "revisionRangeId": "ctxpack-main-last-20",
      "privacyLabel": "source_free_local",
      "base": "main~20",
      "head": "main"
    },
    {
      "name": "RefactoringMiner",
      "path": "../RefactoringMiner",
      "revisionRangeId": "refactoringminer-current-head-20",
      "limit": 30,
      "rankingBudget": 10,
      "baseline": {
        "fileRecallAt10": 0.5186,
        "lexicalBaselineRecallAt10": 0.5008,
        "totalMillis": 265650,
        "gapFamilies": ["lexical_only_miss", "ranked_below_budget"],
        "notes": ["Baseline captured from source-free local E2E evidence."]
      }
    }
  ]
}
```

Fields:

- `manifestVersion`: source-free manifest contract version. v2.3 suites use `ctxpack-benchmark-corpus-v2.3`.
- `name`: source-free suite label used in reports.
- `corpusId`: stable source-free ID for fixed-corpus comparisons.
- `privacyLabel`: expected privacy class for the suite, usually `source_free_local`.
- `description`: optional source-free human context.
- `defaults.limit`: max historical commits per repository.
- `defaults.rankingBudget`: fixed context-file budget K used for combined, lexical, and ablation metrics.
- `defaults.mode`: task type used when replaying commit titles.
- `defaults.targetAgent`: source-free agent label in eval metadata.
- `defaults.semanticEnabled`: explicit opt-in for local semantic retrieval during historical eval.
- `defaults.semanticProvider`: semantic provider used when semantic retrieval is enabled. Defaults to `local_hash`.
- `defaults.semanticModel`: optional provider model override. Benchmark reports resolve provider defaults into effective metadata.
- `defaults.semanticDimensions`: optional provider dimension override. Benchmark reports resolve provider defaults into effective metadata.
- `defaults.cacheEnabled`: reuse source-free stored historical eval reports when the repo/range/options/cache schema are unchanged.
- `defaults.forceRefresh`: recompute and overwrite a cached historical eval report for the same source-free range.
- `defaults.parallelism`: number of historical commit samples to evaluate concurrently. Output ordering remains deterministic.
- `defaults.roleFilters`: documented target roles for this benchmark scope. Phase 9 records these filters in source-free metadata; later v1.2 phases use them for deeper metric segmentation.
- `repositories[*].name`: source-free repo label.
- `repositories[*].path`: local repository path.
- `repositories[*].revisionRangeId`: source-free stable label for the revision range.
- `repositories[*].privacyLabel`: expected repo privacy class.
- `repositories[*].base` / `head`: optional stable revision range.
- `repositories[*].limit`, `rankingBudget`, `mode`, `targetAgent`, `semanticEnabled`, `semanticProvider`, `semanticModel`, `semanticDimensions`, `cacheEnabled`, `forceRefresh`, `parallelism`, `roleFilters`: per-repo overrides.
- `repositories[*].baseline`: optional locked source-free baseline metadata for regression suites. Supported fields are `fileRecallAt10`, `lexicalBaselineRecallAt10`, `totalMillis`, `gapFamilies`, and `notes`.

## Run

```bash
ctxpack eval benchmark --config .ctxpack/benchmarks/retrieval-quality.json --format markdown
ctxpack eval benchmark --config .ctxpack/benchmarks/retrieval-quality.json --format json
ctxpack eval history --repo /path/to/repo --semantic --format json
ctxpack eval history --repo /path/to/repo --semantic --semantic-provider local_fastembed --format json
ctxpack eval history --repo /path/to/repo --cache --parallelism 4 --format markdown
ctxpack eval history --repo /path/to/repo --cache --force --parallelism 4 --format json
ctxpack eval baselines --repo /path/to/repo --limit 20 --budget 10 --format markdown
ctxpack eval compare --base-report previous.json --head-report current.json --threshold fileRecallAt10=0.05
ctxpack eval proof --config .ctxpack/benchmarks/retrieval-quality.json
```

The Markdown report is meant for local inspection. The JSON report is the stable contract for future release gates, portfolio tables, and regression comparison. `ctxpack eval baselines` runs a paired source-free comparison for default ctxpack, lexical, no-context, signal-only, and ablation variants on the same historical corpus. `ctxpack eval compare` consumes two benchmark JSON reports and emits source-free metric deltas, retrieval-gap family deltas, and threshold pass/fail checks. `ctxpack eval proof` runs the configured benchmark suite and emits the adoption-facing proof report with headline metrics, v2.3 fixed-corpus identity, paired baseline verdicts, runtime diagnostics, feature-export privacy, learned-policy status, limitations, when ctxpack helps, when it does not, and future work from measured gaps.

## Privacy Boundary

Benchmark reports include:

- manifest version, corpus ID, privacy labels, and revision range IDs;
- suite and repository labels;
- repo IDs;
- revision range metadata;
- recall and baseline metrics;
- no-context baseline metrics under the same fixed K budget;
- token ROI rows for brief, standard, and deep pack options;
- source-free file paths and role labels;
- skipped path counts and privacy status.
- optional locked baseline deltas for source-free metrics and runtime.
- cache hits/misses, effective parallelism, git sample cost, ranking cost, pack/compiler cost, and slow commits.

Benchmark reports do not include:

- source snippets;
- prompt text;
- commit subjects;
- remote uploads;
- cloud embeddings or reranking.

## Adding A Repo

1. Keep the repository local.
2. Add a named entry to the suite JSON.
3. Prefer a bounded `base` / `head` range for reproducibility.
4. Keep `limit` small while developing the benchmark.
5. Run Markdown first to inspect metrics, then JSON for machine checks.

For large-history proof, RefactoringMiner is the primary v1.2 target. Add a second real repository to avoid overfitting retrieval decisions to one project.

The v2.3 locked RefactoringMiner manifest lives at:

```bash
ctxpack eval benchmark --config .ctxpack/benchmarks/refactoringminer-v23.json --format markdown
```

It intentionally uses a source-free baseline from the May 19 E2E run:

- ctxpack Recall@10: `0.5186`
- lexical baseline Recall@10: `0.5008`
- total historical eval runtime baseline: `265650` ms

Use this as the first large-history regression target, not as a broad product claim.

## v2.5 Multi-Repo Quality Baseline

v2.5 uses the same benchmark-suite command as the default multi-repo proof path:

```bash
ctxpack eval benchmark --config .ctxpack/e2e/v25-multirepo-baseline-config.json --format json
```

The Phase 61 baseline ran RefactoringMiner plus ctxpack itself:

| Repo | Commits | Default Recall@10 | Lexical Recall@10 | Lift@10 |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | 10 | 0.7767 | 0.7792 | -0.0025 |
| ctxpack | 10 | 0.2270 | 0.2742 | -0.0472 |

Interpretation:

- RefactoringMiner is near lexical parity in this bounded run.
- ctxpack trails lexical and exposes repeated docs/scripts/planning and compiler-source gap families.
- Production embeddings, rerankers, or learned policies should not be promoted unless they improve this multi-repo proof under the same source-free privacy boundary.

The concise Phase 61 evidence lives at `.planning/e2e/2026-05-22-v25-multirepo-baseline.md`.

## v2.5 Production Local Embedding Quality

Phase 62 used the Phase 61 two-repo corpus to compare default retrieval,
`local_hash`, and a production local fastembed model. The benchmark reports now
resolve semantic provider metadata in `effectiveConfig`, including
`semanticProvider`, `semanticModel`, `semanticDimensions`,
`semanticProviderRole`, and `semanticQualityBackend`.

Commands:

```bash
cargo run -p ctxpack --features local-embeddings -- \
  eval benchmark --config .ctxpack/e2e/phase62-default-config.json --format json

cargo run -p ctxpack --features local-embeddings -- \
  eval benchmark --config .ctxpack/e2e/phase62-local-hash-config.json --format json

cargo run -p ctxpack --features local-embeddings -- \
  eval benchmark --config .ctxpack/e2e/phase62-local-fastembed-config.json --format json
```

Results:

| Variant | Provider role | Quality backend | RefactoringMiner Recall@10 | ctxpack Recall@10 | Total repo runtime |
| --- | --- | --- | ---: | ---: | ---: |
| Default, semantic off | `deterministic_scaffold` | false | 0.7767 | 0.2299 | 26.3s |
| `local_hash` | `deterministic_scaffold` | false | 0.7767 | 0.2299 | 57.8s |
| `local_fastembed` `AllMiniLML6V2Q` | `production_local` | true | 0.7767 | 0.2299 | 183.7s |

Interpretation:

- `local_fastembed` is source-free, local-only, and usable behind the `local-embeddings` feature.
- The Jina code model is available but too slow for the current full historical eval path.
- The model cache defaults to repo `.ctxpack/cache/fastembed` inside a git repo, otherwise `CTXPACK_HOME/cache/fastembed`.
- The measured fastembed variant matched default recall but did not beat lexical/default retrieval.
- Runtime cost blocks default promotion.
- v2.5 should proceed to reranker/fusion and gap-family work before attempting semantic promotion again.

## Interpreting Metrics

- `rankingComparison.combined`: ctxpack's hybrid context-file ranking at the configured K budget.
- `rankingComparison.lexicalBaseline`: exact/BM25-style local search baseline under the same K budget.
- `rankingComparison.noContextBaseline`: a zero-file baseline that represents an agent starting without ctxpack-provided repository context.
- `signalAblations`: source-free lift checks after removing one retrieval signal family.
- `ctxpack eval baselines`: paired comparison report with thresholded verdicts, lexical parity/regression status, token ROI, validation coverage, signal saturation, runtime, and retrieval-gap taxonomy.
- `tokenRoi`: brief, standard, and deep budget estimates showing useful changed-file targets per 1k estimated context tokens.
- `largerPackAddsLittleValue`: true when a larger budget adds no additional useful changed-file targets over the previous budget in the current fixed ranking.
- `retrievalGapSummaries`: source-free failure families grouped by role, signal gap, package, path family, target status, and recommendation area.
- `ctxpack eval compare`: compares two benchmark reports across recall, token ROI, skipped paths, and gap families; configured thresholds fail when a metric drops more than the allowed amount.
- `ctxpack eval proof`: generates a concise product proof report from a local suite and embeds the underlying source-free benchmark report in JSON output.
- `runtime`: reports total time, per-commit time, overhead, cache hits/misses, effective parallelism, git sample cost, ranking cost, pack/compiler cost, and the slowest source-free commit summaries.

## Query Trace Debugging

When retrieval quality does not improve, inspect `queryTrace` before changing weights. The trace answers whether the benchmark actually exercised the intended path:

- `facets`: source-free extracted evidence such as `explicit_path`, `stack_frame`, `symbol`, `error_text`, `domain_phrase`, `commit_clue`, and `current_diff_path`.
- `retrieverQueries`: the terms sent to lexical, semantic, symbol, graph, history, and test retrieval.
- `fusionControls`: the guardrails used for anchor dominance, exact-evidence protection, semantic cap, and semantic weight.
- `effectiveFilters.semanticProvider`: the selected semantic provider when semantic retrieval is enabled, so cached eval ranges and reports distinguish `local_hash` from `local_fastembed`.
- `sourceTextLogged`: must remain `false`.

If `semanticEnabled` is true but `queryTrace.retrieverQueries.semanticPhrases` contains only weak generic terms, a semantic backend should not be expected to lift Recall@K. Conversely, if explicit paths or stack frames appear in the trace, those anchors should remain protected even when semantic retrieval is enabled.

## v2.4 Semantic/Precision Gate

Phase 60 adds a fixed-corpus gate for semantic, precision, and reranker
variants:

```bash
ctxpack eval gate --repo /path/to/repo --limit 20 --budget 10 --format json
```

The gate emits deterministic variant rows for `lexical_baseline`,
`ctxpack_default`, `local_metadata_reranked`, `local_semantic`,
`precision_enriched_semantic`, `semantic_precision_full_hybrid`, and
`policy_allowed_reranked`. Policy-blocked variants are reported as `skipped`,
not omitted.

The report includes Recall@K, precision proxy, MRR where available, Test
Recall@10, runtime/cache fields, token efficiency, provider policy, precision
status, protected-evidence miss rate, named wins, named regressions, and named
misses.

Protected evidence is source-free metadata for paths that carry explicit anchor,
current-diff, lexical, or symbol signals. Promotion gates treat a variant that
demotes protected paths kept by the default ranking as a named regression. This
prevents semantic, graph, or metadata reranking experiments from hiding exact
evidence behind aggregate Recall@K gains.

Benchmark suites can evaluate the local metadata reranker without changing MCP
tools or default agent behavior:

```json
{
  "defaults": {
    "localMetadataReranker": true
  }
}
```

This switch only affects historical eval ranking. It reorders first-stage
candidates using local source-free metadata such as signal scores, protected
signals, confidence, and evidence count.

Gate decisions:

- `promote`: measured lift clears the quality floor with no named regressions
  or unsafe provider state.
- `hold`: mixed or neutral results; keep the feature opt-in.
- `block`: privacy/policy violation, named regression, or metric regression.

The gate is intentionally conservative. A feature existing is not evidence that
it should become a default.

## v2.5 Phase 63 Local Reranker Gate

Phase 63 compared default ranking with the eval-only local metadata reranker on
the same two-repo corpus used by Phase 62.

```bash
ctxpack eval benchmark --config .ctxpack/e2e/phase62-default-config.json --format json
ctxpack eval benchmark --config .ctxpack/e2e/phase63-local-reranker-config.json --format json
ctxpack eval gate --limit 5 --budget 10 --format json
```

Results:

| Repo | Default Recall@10 | Reranked Recall@10 | Delta | Default MRR@K | Reranked MRR@K | Test Recall@10 delta | Protected miss-rate delta | Runtime delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 0.1375 | 0.6642 | +0.5267 | 0.1500 | 0.6125 | +1.0000 | +0.1509 | +13.4s |
| ctxpack | 0.2049 | 0.1927 | -0.0122 | 0.6333 | 0.7167 | +0.5000 | +0.0000 | -0.6s |

Decision: hold/block default promotion. The reranker produced a large
RefactoringMiner lift, but it regressed ctxpack Recall@10 and the gate named
protected-evidence demotions. The next work is Phase 64 gap-family retrieval
improvements, not default reranker promotion.

## Product Proof

The product proof is intentionally narrow. It does not claim universal agent improvement. It answers:

- what local benchmark suite was run;
- which fixed corpus ID, manifest version, and privacy label define the run;
- how many repositories and commits were evaluated;
- headline recall, baseline lift, test recall, and token ROI metrics;
- paired lexical-baseline verdicts for each evaluated repository;
- total historical-eval runtime across the proof run;
- whether feature export remained local-only and source-free;
- whether learned retrieval policy is available only behind thresholded status;
- when ctxpack helps;
- when ctxpack does not help;
- limitations in the benchmark design;
- which future milestone should address repeated gap families.

Useful context at lexical parity is not world-class lift. If ctxpack matches
lexical retrieval, the proof can still show product usefulness around related
tests, token ROI, privacy-safe diagnostics, and process guidance, but it should
not be described as state-of-the-art retrieval. World-class claims require
repeated lift on fixed corpora, paired baseline verdicts that clear thresholds,
runtime that stays acceptable, and process-level context metrics from real
agent sessions.

The release gate can run this proof optionally:

```bash
CTXPACK_BENCHMARK_CONFIG=/absolute/path/to/retrieval-quality.json bash scripts/release-gate.sh
```

When enabled, the gate fails if the proof cannot be generated, if the
proof/embedded benchmark privacy status is not local-only, if the v2.3 summary
is missing fixed corpus identity or paired baseline verdict fields, if
feature-export privacy regresses, if learned-policy status allows silent
defaults, or if proof-boundary language is missing.

The deterministic release smoke is:

```bash
bash scripts/smoke-v23-eval.sh
```

It creates a small local git repository and checks source-free feature export,
feedback recording, offline learned policy, paired baselines, runtime
diagnostics, and product proof. RefactoringMiner and multi-repo suites remain
optional external gates:

```bash
ctxpack eval benchmark --config .ctxpack/benchmarks/refactoringminer-v23.json --format markdown
CTXPACK_BENCHMARK_CONFIG="$(pwd)/.ctxpack/benchmarks/refactoringminer-v23.json" bash scripts/release-gate.sh
```

If the external checkout is not available, skip with the reason "external corpus unavailable." The skip is acceptable for the default release gate; it is
not evidence that the large-history proof passed.
