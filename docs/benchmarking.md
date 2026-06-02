# Retrieval Benchmarking

ctxhelm v1.2 uses source-free benchmark suites to answer the product question:

> Does ctxhelm help an agent choose better files and tests than native search alone?

The benchmark runner reuses `ctxhelm eval history` for each configured repository. It records file/test recall, lexical and no-context baseline comparison, signal ablations, token ROI, grouped retrieval gaps, skipped path counts, runtime diagnostics, and privacy status without storing source snippets, prompt text, or commit subjects.

Historical eval reports keep two changed-file views:

- `safeChangedFiles`: the full source-free safe patch surface after generated, sensitive, deleted, and otherwise excluded paths are removed.
- `retrievalTargetFiles`: the subset of safe changed files that existed in the parent snapshot and could therefore be retrieved as context before the patch.

Recall, MRR, token ROI, missing-file analysis, and gap summaries use `retrievalTargetFiles`. This avoids treating newly-created files as retrieval failures while still preserving the full changed-file audit trail.

The product-proof release gate evaluates two channels:

- **Context channel:** non-test `retrievalTargetFiles` are compared against the lexical baseline at K=10.
- **Validation channel:** changed tests are measured through `recommendedTests` and targeted commands.

This matches the product contract: ctxhelm returns source/docs/config as task context and tests as validation context. All-file recall remains in reports for transparency, but default promotion is decided by context lift plus validation-test recall so tests are not double-counted as both target files and validation commands.

Product-proof corpus verdicts also expose machine-checkable divergence fields:

- `contextVsAllFileDeltaAt10`: ctxhelm context-channel Recall@10 minus all-file Recall@10.
- `lexicalContextVsAllFileDeltaAt10`: lexical context-channel Recall@10 minus lexical all-file Recall@10.
- `allFileDivergenceExplained`: `true` only when any all-file lexical deficit is explained by non-regressed context recall plus covered validation targets.
- `sourceRecallAt10`: ctxhelm Recall@10 for source-code targets.
- `lexicalSourceRecallAt10`: lexical baseline Recall@10 for source-code targets.
- `sourceDeltaAt10`: ctxhelm source Recall@10 minus lexical source Recall@10.

The release gate and `scripts/check-product-proof.py` block stale proof JSON that
does not contain these source-recall fields, unexplained all-file lexical
deficits, and source-code recall regressions beyond the release tolerance. This
keeps the proof honest when lexical wins raw all-file recall by ranking
validation tests as files, while still rejecting aggregate wins that hide worse
source-code target selection.

Product-proof JSON also includes `releaseGate.lexicalComparison`, a suite-level
summary of the same boundary. `allFileClaim` reports whether ctxhelm beats,
matches, or trails lexical when every safe changed file is counted together.
The raw all-file counters remain visible, and the summary also splits raw
trailing corpora into explained and unexplained trails. An explained all-file
trail means the corpus lost only in the mixed file channel while context recall
and validation coverage were non-regressed; it is counted as match-like for the
headline `allFileClaim` but still exposed as a raw trail for auditability.
`agentEvidenceClaim` reports the same comparison across the actual evidence set
ctxhelm gives an agent: selected context files plus related tests and validation
commands. This is the production adoption claim because coding agents consume
both context and verification guidance. In the same summary, `contextClaim`
reports the comparison after validation targets are removed from the context
channel. The three claims prevent production notes from overclaiming
repository-wide target-file wins when the measured claim is narrower: selected
agent evidence plus separately reported target-file and context-channel scores.

The latest clean four-repo proof also reports zero protected target misses
after broad operational floors for root governance docs, exact config matches,
and workflow lifecycle scripts. The current summary is `allFileClaim = mixed`
with beat `3`, raw match `0`, raw trail `1`, explained trail `1`, unexplained
trail `0`, average File Recall@10 `0.61190045` versus lexical `0.45709258`,
average file delta `+0.15480787`, average agent-evidence delta `+0.2570628`,
and average context delta `+0.30652046`.

v2.3 treats benchmark suites as fixed corpus manifests. Older suite files still work, but v2.3+ manifests should include a manifest version, corpus ID, privacy label, revision range ID, and optional locked baseline metadata so quality claims are reproducible.

Phase 58 adds source-free query construction traces to prepare-task and historical eval commit rows. These traces record extracted paths, stack frames, symbols, error terms, domain terms, commit clues, retriever query sets, and fusion controls. They intentionally store task hashes and bounded facets instead of raw prompts or source snippets.

Historical eval also records broad multi-area task pressure. `broadScopeTask`
marks commits whose prompt or changed-file surface spans workflow/eval/lint
style changes that cannot be fully covered by a small K=10 context budget, and
`broadScopeCommitCount` summarizes that count at report level. These fields are
source-free diagnostics; they do not remove targets from recall denominators.

Historical eval reports also include source-free `graphEdgeProfiles`. These
profiles split graph candidate evidence by edge label, such as `imports`,
`python_reexport`, or `precision:calls`, and report candidate count,
selected-at-10 count, retrieval-target count, target hits, and target misses.
This keeps GraphRAG work measurable by edge family without changing the default
ranking or adding source text to reports.

Reports also include `graphEdgeAblations`. These are conservative diagnostic
ablations: disabling an edge label removes a selected file only when that file's
only evidence is the disabled dependency edge family. Files that also have
lexical, symbol, test, history, co-change, or another graph edge label remain in
the ranking. This measures edge-family lift without overstating impact from
files that have independent retrieval support.

Source dependency floors use those diagnostics conservatively when budget is
tight. Dependency edge labels are ordered by expected precision before raw edge
confidence: `precision:*` edges first, then direct `imports`, then other
dependency labels, then `python_reexport`. This does not expand graph depth or
increase context budget; it only chooses which graph-supported source neighbor
survives when the dependency reserve cannot include every candidate.

Prepare-task plans now expose `contextAreas` for broad multi-area prompts. This
is an additive, source-free channel that groups candidate paths by repository
area, reports how many candidate and selected paths each area contributed, and
lists representative paths, concrete unselected `nextReadPaths`, source-free
`roleCounts`, source-free `selectedRoleCounts`, source-free `signalCounts`, and
`unselectedCount`. Docs areas are included in this channel. It lets agents
inspect likely adjacent source/docs areas with native file reads without
forcing those areas into the protected top-10 file budget, so retrieval quality
metrics remain comparable to earlier proofs.

`signalCounts` are per-area counts of candidate files by retrieval signal
family. They are deduplicated per candidate and can include `lexical`,
`lexical_expansion`, `symbol`, `dependency`, `related_test`, `semantic`,
`co_change`, `current_diff`, `history`, `docs`, `config`, `anchor`, and
`memory`. The counts explain why an area was surfaced without storing source
text or changing the recall denominator.

Retrieval gap summaries reuse the same source-free area profile when a missed
target belongs to a task-conditioned context area. Profiled gap summaries expose
`contextAreaSignalCounts`, `contextAreaRoleCounts`,
`contextAreaSelectedRoleCounts`, and `contextAreaUnselectedCount`. These fields
make proof gaps actionable without storing source text or requiring a manual
join against individual commit `contextAreas`.

Dynamic MCP context-area resources expose a separate inventory-wide scope.
`resourceScope.kind = safeInventoryArea`, `taskConditioned = false`,
`countsSource = safeInventory`, and `pathSource = safeInventory` distinguish
resource-level role counts and read batches from task-conditioned
`contextAreas` in a prepared plan.

## Suite File

Benchmark suites are JSON files. Paths may be absolute or relative to the suite file.

```json
{
  "manifestVersion": "ctxhelm-benchmark-corpus-v2.3",
  "name": "retrieval-quality-smoke",
  "corpusId": "ctxhelm-local-retrieval-quality-smoke",
  "privacyLabel": "source_free_local",
  "description": "Bounded source-free retrieval benchmark over local repos",
  "defaults": {
    "limit": 20,
    "rankingBudget": 10,
    "mode": "bug_fix",
    "targetAgent": "codex",
    "semanticEnabled": false,
    "semanticProvider": "local_hash",
    "semanticModel": "ctxhelm-local-hash-v1",
    "semanticDimensions": 64,
    "lexicalBackendComparison": false,
    "cacheEnabled": true,
    "forceRefresh": false,
    "parallelism": 4,
    "roleFilters": ["source", "test"]
  },
  "repositories": [
    {
      "name": "ctxhelm",
      "path": ".",
      "revisionRangeId": "ctxhelm-main-last-20",
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

- `manifestVersion`: source-free manifest contract version. v2.3 suites use `ctxhelm-benchmark-corpus-v2.3`.
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
- `defaults.lexicalBackendComparison`: optional source-free BM25-vs-legacy lexical backend corpus comparison. Defaults to `false` because it runs both lexical backends over the selected historical tasks.
- `defaults.cacheEnabled`: reuse source-free stored historical eval reports when the repo/range/options/cache schema are unchanged.
- `defaults.forceRefresh`: recompute and overwrite a cached historical eval report for the same source-free range.
- `defaults.parallelism`: number of historical commit samples to evaluate concurrently. Output ordering remains deterministic.
- `defaults.roleFilters`: documented target roles for this benchmark scope. Phase 9 records these filters in source-free metadata; later v1.2 phases use them for deeper metric segmentation.
- `repositories[*].name`: source-free repo label.
- `repositories[*].path`: local repository path.
- `repositories[*].revisionRangeId`: source-free stable label for the revision range.
- `repositories[*].privacyLabel`: expected repo privacy class.
- `repositories[*].base` / `head`: optional stable revision range.
- `repositories[*].limit`, `rankingBudget`, `mode`, `targetAgent`, `semanticEnabled`, `semanticProvider`, `semanticModel`, `semanticDimensions`, `lexicalBackendComparison`, `cacheEnabled`, `forceRefresh`, `parallelism`, `roleFilters`: per-repo overrides.
- `repositories[*].proofRuntimeCeilingMillis`: optional source-free product-proof
  runtime ceiling override for a specific repository. Omit it to keep the
  default `5000ms` per-commit promotion gate; use it only when a detached
  large-history fixture has measured cold-start cost that should not weaken the
  global release threshold.
- `repositories[*].baseline`: optional locked source-free baseline metadata for regression suites. Supported fields are `fileRecallAt10`, `lexicalBaselineRecallAt10`, `totalMillis`, `gapFamilies`, and `notes`.

## Run

```bash
ctxhelm eval benchmark --config .ctxhelm/benchmarks/retrieval-quality.json --format markdown
ctxhelm eval benchmark --config .ctxhelm/benchmarks/retrieval-quality.json --format json
ctxhelm eval history --repo /path/to/repo --semantic --format json
ctxhelm eval history --repo /path/to/repo --semantic --semantic-provider local_fastembed --format json
ctxhelm eval history --repo /path/to/repo --cache --parallelism 4 --format markdown
ctxhelm eval history --repo /path/to/repo --cache --force --parallelism 4 --format json
ctxhelm eval baselines --repo /path/to/repo --limit 20 --budget 10 --format markdown
ctxhelm eval compare --base-report previous.json --head-report current.json --threshold fileRecallAt10=0.05
ctxhelm eval proof --config .ctxhelm/benchmarks/retrieval-quality.json
```

The Markdown report is meant for local inspection. The JSON report is the stable contract for future release gates, portfolio tables, and regression comparison. `ctxhelm eval baselines` runs a paired source-free comparison for default ctxhelm, lexical, no-context, signal-only, and ablation variants on the same historical corpus. `ctxhelm eval compare` consumes two benchmark JSON reports and emits source-free metric deltas, retrieval-gap family deltas, and threshold pass/fail checks. `ctxhelm eval proof` runs the configured benchmark suite and emits the adoption-facing proof report with headline metrics, v2.3 fixed-corpus identity, paired baseline verdicts, optional lexical backend comparison evidence, runtime diagnostics, feature-export privacy, learned-policy status, limitations, when ctxhelm helps, when it does not, and future work from measured gaps.

## Real-Corpus Fixture Health

Large-history proof runs should use a clean detached fixture instead of an
ambient sibling checkout. Before measuring RefactoringMiner or another
large-history repository, prepare and validate the corpus with:

```bash
bash scripts/prepare-benchmark-corpus.sh \
  --source https://github.com/tsantalis/RefactoringMiner.git \
  --revision e319af8d6b51d821b61d2f735ad211631775adfb \
  --worktree ../ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --min-commits 20 \
  --output .ctxhelm/e2e/refactoringminer-corpus-health.json \
  --refresh
```

The report is intentionally source-free. It records revision identity, commit
count, dirty-file count, object-store connectivity, history usability, refresh
status, and privacy metadata while omitting source snippets, commit subjects,
diffs, terminal logs, and prompts. A `ready` report means the fixture is clean
enough for benchmark configs or direct commands such as:

```bash
ctxhelm eval lexical corpus \
  --repo ../ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 20 \
  --budget 10 \
  --format json
```

If the report is `blocked`, treat the proof as environment-blocked rather than
as retrieval-quality evidence. Dirty, corrupt, missing-history, or wrong-revision
fixtures are not acceptable product-proof inputs.

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
ctxhelm eval benchmark --config .ctxhelm/benchmarks/refactoringminer-v23.json --format markdown
```

It intentionally uses a source-free baseline from the May 19 E2E run:

- ctxhelm Recall@10: `0.5186`
- lexical baseline Recall@10: `0.5008`
- total historical eval runtime baseline: `265650` ms

Use this as the first large-history regression target, not as a broad product claim.

## v2.5 Multi-Repo Quality Baseline

v2.5 uses the same benchmark-suite command as the default multi-repo proof path:

```bash
ctxhelm eval benchmark --config .ctxhelm/e2e/v25-multirepo-baseline-config.json --format json
```

The Phase 61 baseline ran RefactoringMiner plus ctxhelm itself:

| Repo | Commits | Default Recall@10 | Lexical Recall@10 | Lift@10 |
| --- | ---: | ---: | ---: | ---: |
| RefactoringMiner | 10 | 0.7767 | 0.7792 | -0.0025 |
| ctxhelm | 10 | 0.2270 | 0.2742 | -0.0472 |

Interpretation:

- RefactoringMiner is near lexical parity in this bounded run.
- ctxhelm trails lexical and exposes repeated docs/scripts/planning and compiler-source gap families.
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
cargo run -p ctxhelm --features local-embeddings -- \
  eval benchmark --config .ctxhelm/e2e/phase62-default-config.json --format json

cargo run -p ctxhelm --features local-embeddings -- \
  eval benchmark --config .ctxhelm/e2e/phase62-local-hash-config.json --format json

cargo run -p ctxhelm --features local-embeddings -- \
  eval benchmark --config .ctxhelm/e2e/phase62-local-fastembed-config.json --format json
```

Results:

| Variant | Provider role | Quality backend | RefactoringMiner Recall@10 | ctxhelm Recall@10 | Total repo runtime |
| --- | --- | --- | ---: | ---: | ---: |
| Default, semantic off | `deterministic_scaffold` | false | 0.7767 | 0.2299 | 26.3s |
| `local_hash` | `deterministic_scaffold` | false | 0.7767 | 0.2299 | 57.8s |
| `local_fastembed` `AllMiniLML6V2Q` | `production_local` | true | 0.7767 | 0.2299 | 183.7s |

Interpretation:

- `local_fastembed` is source-free, local-only, and usable behind the `local-embeddings` feature.
- The Jina code model is available but too slow for the current full historical eval path.
- The model cache defaults to repo `.ctxhelm/cache/fastembed` inside a git repo, otherwise `CTXHELM_HOME/cache/fastembed`.
- The measured fastembed variant matched default recall but did not beat lexical/default retrieval.
- Runtime cost blocks default promotion.
- v2.5 should proceed to reranker/fusion and gap-family work before attempting semantic promotion again.

## Interpreting Metrics

- `rankingComparison.combined`: ctxhelm's hybrid context-file ranking at the configured K budget.
- `rankingComparison.lexicalBaseline`: exact/BM25-style local search baseline under the same K budget.
- `rankingComparison.noContextBaseline`: a zero-file baseline that represents an agent starting without ctxhelm-provided repository context.
- `signalAblations`: source-free lift checks after removing one retrieval signal family.
- `ctxhelm eval baselines`: paired comparison report with thresholded verdicts, lexical parity/regression status, token ROI, validation coverage, signal saturation, runtime, and retrieval-gap taxonomy.
- `tokenRoi`: brief, standard, and deep budget estimates showing useful changed-file targets per 1k estimated context tokens.
- `largerPackAddsLittleValue`: true when a larger budget adds no additional useful changed-file targets over the previous budget in the current fixed ranking.
- `retrievalGapSummaries`: source-free failure families grouped by role, signal gap, package, path family, target status, and recommendation area.
- `ctxhelm eval compare`: compares two benchmark reports across recall, token ROI, skipped paths, and gap families; configured thresholds fail when a metric drops more than the allowed amount.
- `ctxhelm eval proof`: generates a concise product proof report from a local suite and embeds the underlying source-free benchmark report in JSON output.
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
ctxhelm eval gate --repo /path/to/repo --limit 20 --budget 10 --format json
```

Use `--semantic-provider` to evaluate a specific local semantic backend in the
gate:

```bash
ctxhelm eval gate --repo /path/to/repo --limit 20 --budget 10 \
  --semantic-provider local_fastembed --format json
```

`local_hash` remains the deterministic scaffold. `local_fastembed` is the
production-local backend and requires a build compiled with the
`local-embeddings` feature.

The gate emits deterministic variant rows for `lexical_baseline`,
`ctxhelm_default`, `local_metadata_reranked`, `local_semantic`,
`precision_enriched_semantic`, `semantic_precision_full_hybrid`, and
`policy_allowed_reranked`. Policy-blocked variants are reported as `skipped`,
not omitted.

The report includes Recall@K, precision proxy, MRR where available, Test
Recall@10, runtime/cache fields, token efficiency, provider policy, precision
status, protected-evidence miss rate, semantic contribution summary, named wins,
named regressions, and named misses.

The semantic contribution summary also emits source-free diagnostics:

- `semantic_contribution_no_candidates`: the selected semantic provider produced
  no candidate files in the gate run.
- `semantic_contribution_no_unique_target_hits`: semantic selected target files,
  but none were unique beyond the lexical baseline top K.
- `semantic_contribution_unique_target_hits`: semantic contributed target files
  absent from the lexical baseline top K.
- `semantic_contribution_unique_non_targets`: semantic contributed files absent
  from the lexical baseline top K, but those unique semantic files were not
  retrieval targets. This is a noise/coupling diagnostic, not promotion
  evidence.
- `semantic_contribution_missed_targets_coupled`: semantic missed target files
  that already had source-free graph, history, or symbol signal. Treat this as a
  graph/fusion ordering problem before adding more embedding text.
- `semantic_contribution_missed_targets_no_signal`: semantic missed target files
  that also lacked lexical, graph, history, and area signal. Treat this as a
  semantic document/query construction or index coverage problem.

The `semanticContribution.semanticMissedTargetGapFamilies` JSON field groups
semantic-missed retrieval targets into source-free families such as
`semantic_miss_area_context_only`,
`semantic_miss_nonsemantic_coupled_signal`, and
`semantic_miss_no_candidate_signal`.

Phase 176 turns the `area_context_only` diagnosis into a better progressive
read surface for focused tasks. When a standard plan already selects
source-like files in an area and has unselected source-like candidates nearby,
the plan now emits a focused context area such as
`ctxhelm://repo/context-area/src%2Fmain`. This does not promote un-signaled
files into the top-K context ranking. It gives agents a source-free area
resource to inspect after the target list proves too narrow. Eval ranking keeps
validation-test reserves tied to task broadness, not to whether context-area
guidance is present, so focused area hints do not silently displace tests from
Recall@10 accounting.

Phase 177 makes those area resources more precise for JVM repositories.
`src/main|test/java|kotlin` paths group by source root plus package components
instead of collapsing to `src/main` or `src/test`. The RefactoringMiner
`UMLClassBaseDiff.java` area-only miss now points at
`ctxhelm://repo/context-area/src%2Fmain%2Fjava%2Fgr%2Fuom%2Fjava%2Fxmi`,
preserving Recall@10 while giving agents a narrower progressive read resource.

Protected evidence is source-free metadata for budgeted paths that carry
explicit anchor, current-diff, lexical, or symbol signals. The protected set is
bounded by the eval context budget, so broader candidate generation can add
`lexical_expansion` evidence without making the protected-evidence requirement
impossible to satisfy. Promotion gates treat a variant that demotes protected
paths kept by the default ranking as a named regression. This prevents semantic,
graph, lexical-expansion, or metadata reranking experiments from hiding exact
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
ctxhelm eval benchmark --config .ctxhelm/e2e/phase62-default-config.json --format json
ctxhelm eval benchmark --config .ctxhelm/e2e/phase63-local-reranker-config.json --format json
ctxhelm eval gate --limit 5 --budget 10 --format json
```

Results:

| Repo | Default Recall@10 | Reranked Recall@10 | Delta | Default MRR@K | Reranked MRR@K | Test Recall@10 delta | Protected miss-rate delta | Runtime delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 0.1375 | 0.6642 | +0.5267 | 0.1500 | 0.6125 | +1.0000 | +0.1509 | +13.4s |
| ctxhelm | 0.2049 | 0.1927 | -0.0122 | 0.6333 | 0.7167 | +0.5000 | +0.0000 | -0.6s |

Decision: hold/block default promotion. The reranker produced a large
RefactoringMiner lift, but it regressed ctxhelm Recall@10 and the gate named
protected-evidence demotions. The next work is Phase 64 gap-family retrieval
improvements, not default reranker promotion.

## Product Proof

The product proof is intentionally narrow. It does not claim universal agent improvement. It answers:

- what local benchmark suite was run;
- which fixed corpus ID, manifest version, and privacy label define the run;
- how many repositories and commits were evaluated;
- headline recall, baseline lift, test recall, and token ROI metrics;
- paired lexical-baseline verdicts for each evaluated repository;
- beat/match/trail release-gate status per corpus and retrieval variant;
- whether the default retrieval mode is allowed to ship from the configured
  proof;
- total historical-eval runtime across the proof run;
- whether feature export remained local-only and source-free;
- whether learned retrieval policy is available only behind thresholded status;
- when ctxhelm helps;
- when ctxhelm does not help;
- limitations in the benchmark design;
- which future milestone should address repeated gap families.

Useful context at lexical parity is not world-class lift. If ctxhelm matches
lexical retrieval, the proof can still show product usefulness around related
tests, token ROI, privacy-safe diagnostics, and process guidance, but it should
not be described as state-of-the-art retrieval. World-class claims require
repeated lift on fixed corpora, paired baseline verdicts that clear thresholds,
runtime that stays acceptable, and process-level context metrics from real
agent sessions.

### v2.5 Release-Proof Status

Phase 65 adds a machine-checkable `releaseGate` section to product proof JSON
and Markdown. Each required corpus receives a verdict for the configured
retrieval variant:

- `beat`: ctxhelm exceeds lexical Recall@10 by more than the proof threshold.
- `match`: ctxhelm is within the proof threshold of lexical.
- `trail`: ctxhelm falls behind lexical by more than the proof threshold.
- `insufficient_evidence`: the repository failed or produced no eval report.

The current v2.5 fixed two-repo proof promotes default local retrieval under
the channel-aware release gate. Context recall is evaluated on non-test target
context, and validation-test recall is evaluated through the dedicated
`recommended_tests` channel plus broad validation-command coverage:

| Corpus | Variant | Status | ctxhelm Recall@10 | Lexical Recall@10 | Delta | Test Recall@10 | Effective validation recall |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | `ctxhelm_default` | `beat` | 0.778 | 0.741 | +0.037 | 1.000 | 1.000 |
| ctxhelm | `ctxhelm_default` | `beat` | 0.423 | 0.352 | +0.070 | 0.000 | 0.000 |

Phase 74 adds protected-evidence diagnostics to this proof. The original
overall protected miss-rate remains visible, but the report now also separates
retrieval-target protected misses from non-target protected pressure. On the
current required proof after Phase 77, RefactoringMiner has protected target
miss-rate@10 0.059 and ctxhelm has 0.083. The broader fixed-corpus fixture
now promotes after Phase 78: VeriSchema beats through effective validation
recall, and RefactoringMiner is accepted as a safe lexical-ceiling match because
both ctxhelm and lexical have perfect context recall with zero protected
retrieval-target misses. Phase 79 adds protected target floors: the latest
broader proof keeps the gate promoted, drops VeriSchema protected target
miss-rate to `0.0`, and leaves ctxhelm with one residual protected
source-symbol miss.

Recommendation today:

- Use `ctxhelm_default` through MCP/agent-native integrations for progressive
  task plans, related tests, source-free diagnostics, and graph/history context.
- It is valid to claim default local retrieval beats lexical on the fixed
  channel-aware two-repo proof. Do not generalize that to every repository or
  every evidence channel; all-file recall, protected evidence pressure, and
  parser/precision gaps still need follow-up.
- Phase 71 dampens historical planning archive artifacts so they stay searchable
  without crowding active source/current planning evidence. On the current
  ctxhelm history, protected miss-rate@10 improved from 0.250 to 0.163.
- Phase 74 makes protected-evidence pressure source-free and target-aware, so
  follow-up ranking work can focus on protected retrieval-target misses instead
  of total non-target exact/symbol candidates.
- Phase 75 restores parent-bounded co-change history in archive-based eval
  snapshots and reserves co-changed validation tests. This improves the required
  ctxhelm protected target miss-rate.
- Phase 76 narrows partial snapshot history to validation-test discovery in
  historical eval, enriches co-changed tests with runnable commands, and improves
  VeriSchema's broader validation-test Recall@10 from `0.661` to `0.709`
  without perturbing required non-test context promotion.
- Phase 77 adds broad validation fallback commands and effective validation
  recall. VeriSchema's broader raw Test Recall@10 remains `0.709`, but broad
  `pytest` commands raise effective validation recall to `1.000`.
- Phase 78 makes the product-proof gate ceiling-aware. The broader four-repo
  proof now promotes while preserving protected target miss diagnostics for
  ctxhelm and VeriSchema.
- Phase 79 reserves bounded source/config/governance floors and defers planning
  archive artifacts. Required RefactoringMiner and broader VeriSchema protected
  target miss-rates are now `0.0`; ctxhelm still has residual source-symbol
  misses to investigate.
- Phase 80 fixes symbol-floor duplicate accounting. Required and broader
  product proofs now promote with protected retrieval-target miss-rate `0.0`
  across measured corpora.
- Phase 81 fixes cache-hit runtime reporting. Warm cached proof reports the
  cache lookup runtime with zero commit-loop and slow-commit timings, while
  preserving the cached source-free quality metrics.
- Phase 82 makes warm-cache runtime enforceable. Product proofs block cached
  reports that mix cache misses, retain stale cold timings, retain slow-commit
  diagnostics, or exceed `1000ms` warm lookup runtime.
- Phase 83 makes context-vs-all-file divergence source-free and enforceable.
  Product-proof verdicts now include context-vs-all-file deltas and
  `allFileDivergenceExplained`; the checker fails if an all-file lexical
  deficit is not explained by the separate context and validation channels.
- Phase 84 adds `multi_area_task` diagnostics, source-free broad-scope eval
  fields, and a bounded dependency source floor that only activates for broad
  workflow/eval/lint tasks. The broader proof still promotes and improves
  VeriSchema Source Recall@10 from `0.249` to `0.304`.
- Phase 85 adds `contextAreas` to broad multi-area prepare-task plans and packs.
  The field improves agent inspection guidance without changing top-10 ranking:
  broad fixed-corpus quality metrics stayed flat and the warm-cache proof still
  promotes.
- Phase 86 adds bounded Python package re-export graph coverage. Python
  `from package import Symbol` can now resolve `package/__init__.py` and
  re-exported modules as dependency candidates. The broader proof stays
  Recall@10-flat, which keeps the remaining VeriSchema gap focused on
  selection/budget pressure rather than only missing graph candidates.
- Phase 87 keeps retrieval-gap diagnostics aligned with the validation channel.
  Specific Java selectors such as `./gradlew test --tests <FQCN>` and
  `mvn -Dtest=<ClassName> test` now count as validation coverage, and
  validation-covered test files no longer appear as unresolved test-mapping
  gap summaries. RefactoringMiner validation-command recall improves from
  `0.0` to `1.0` with no recall or protected-target regression in the broader
  proof.
- Phase 88 adds bounded source-area inventory candidates for broad multi-area
  tasks after graph/test seed selection. This improves VeriSchema File
  Recall@10 from `0.17936651` to `0.18449473` and Source Recall@10 from
  `0.30409357` to `0.31067252` without raw test, effective validation, or
  protected retrieval-target regression.
- Phase 89 replaces full source re-hashing on inventory cache-hit freshness
  checks with metadata manifest comparison. The pinned broader release proof
  promotes with Phase 88 quality metrics preserved: RefactoringMiner `8279ms`,
  ctxhelm `8317ms`, ReAgent `4264ms`, and VeriSchema `6590ms`.
- Phases 90-101 move broad-area evidence from diagnostics toward release
  contract: the packaged release gate can run the four-repo proof, broad
  context areas are measured, exposed as MCP resources, enriched with
  next-read batches, and current reachable retrieval-gap summaries must retain
  `ctxhelm://repo/context-area/...` URIs plus bounded `nextReadPaths` for the
  product-proof checker to pass.
- Phase 102 closes the wrong-cwd consumption gap for those resources. After an
  explicit-repo MCP tool call, repo-scoped resources fall back to that repo, and
  the deterministic MCP protocol smoke reads context-area resources plus
  `nextReadBatches` from a server cwd outside the workspace.
- Phase 103 adds pinned broad fixed-corpus floors to
  `scripts/check-product-proof.py` for
  `phase92-area-aware-gap-taxonomy-2026-05-31`. This blocks reports that still
  promote overall but regress known-good four-repo metrics. The rejected
  dependency-priority ranking experiment would have dropped VeriSchema File
  Recall@10 from `0.18449473` to `0.17936651`, and now fails the checker.
- Phase 104 adds source-free `nextReadPaths` and `unselectedCount` to broad
  `contextAreas`, includes docs candidates in context-area guidance, and renders
  explicit `Next reads` in generated packs. The product-proof checker now fails
  cleanly when a benchmark repository has no embedded report. The available
  three-repo proof promotes; the full four-repo proof was not claimed because
  the local RefactoringMiner checkout timed out during `git rev-list`.
- Phase 117 adds source-free `roleCounts` and `selectedRoleCounts` to broad
  plan-level `contextAreas`, and renders those role signals into generated packs
  so agents can distinguish source-heavy, validation-heavy, and docs-only areas
  before native reads.
- Phase 118 adds source-free `resourceScope` metadata to context-area MCP
  resources so agents can distinguish safe-inventory resource counts from
  task-conditioned plan `roleCounts` and `selectedRoleCounts`.
- Phase 123 adds source-free `coverageProfile` metadata to context-area MCP
  resources. The profile classifies an area as implementation, validation,
  docs, or a mixed shape, and names the recommended first read batch without
  changing target-file ranking.
- Phase 105 makes history-unavailable benchmark repositories machine-checkable.
  If git history sampling fails or times out, historical eval emits an embedded
  zero-commit report, benchmark output records a source-free history-unavailable
  error, and product proof blocks that corpus as `insufficient_evidence` instead
  of producing `report: null`. Degraded zero-commit reports are not cached.
- Keep `local_metadata_reranked` eval-only until named regressions and protected
  evidence behavior clear the gate.
- Keep `local_fastembed` opt-in for experiments and conceptual queries; it is
  local-only but not a default quality win.
- Phase 163 reduces fresh-process `local_fastembed` document-embedding overhead
  by reusing persisted source-free vectors. On the RefactoringMiner proof
  fixture, a bounded 16-vector seed produced `storedVectorCount: 16`; the query
  `Improvement in TypeScriptVisitor` ranked
  `src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java` first and
  improved direct semantic search from `10.56s` with an empty store to `6.03s`
  with the seeded store. Query/model initialization still dominates the warm
  path, so this is a runtime improvement, not yet a semantic-only recall win.
- Phase 164 adds persisted-vector candidates outside the lexical prefilter and
  write-through caching for embedded candidate misses. A full 647-file
  RefactoringMiner `local_fastembed` seed was stopped after more than 9 minutes,
  so foreground default seeding is bounded to 16 vectors. On the same
  `Improvement in TypeScriptVisitor` query, first search after a 16-vector seed
  took `31.48s`, persisted candidate misses, and the second fresh-process search
  took `20.86s`; both ranked `TypeScriptVisitor.java` first and storage grew
  from `16` to `31` vectors without pruning.
- Phase 165 makes failed semantic vector indexing loud instead of reporting a
  successful zero-vector store, fixes documented fastembed model-id mapping for
  `AllMiniLML6V2Q`, and makes `AllMiniLML6V2Q` the default `local_fastembed`
  model. On RefactoringMiner, the AllMini default created 16 vectors, ranked
  `TypeScriptVisitor.java` first, and reduced the second fresh-process search
  from the Jina proof's `20.86s` to `16.92s`.
- Phase 166 adds source-free persisted query-vector reuse and makes
  `local_fastembed` global stored-candidate expansion single-pass. On the same
  RefactoringMiner probe, the second fresh-process AllMini search dropped from
  Phase 165 `16.92s` to `12.08s` while preserving `TypeScriptVisitor.java` as
  the top result.
- Phase 167 removes the shared large-fixture setup bottleneck by pruning
  generated fixture/cache/build directories before inventory and freshness walks
  descend into them. On the clean RefactoringMiner fixture, generated exclusions
  dropped from `38,242` walked files to `25` counted generated files, fresh
  lexical setup took `3.70s`, cache-hit lexical search took `0.08s`, semantic
  status took `0.10s`, the bounded AllMini semantic seed dropped from Phase 166
  `55.65s` to `5.18s`, and the second fresh-process semantic search dropped
  from `12.08s` to `0.11s` while preserving `TypeScriptVisitor.java` as the top
  result. Semantic still remains opt-in until quality gates show target-file
  recall lift, but latency is no longer blocked by generated fixture traversal
  on this proof repo.
- Phase 168 adds source-free identifier aliases to semantic document/query text
  and reports semantic-only non-targets. On the clean RefactoringMiner 3-commit
  `local_fastembed` gate, semantic and lexical/default File Recall@10 both stayed
  at `0.72222227`, semantic-only target hits stayed `0`, and the only
  semantic-only file was the non-target
  `src/test/java/org/refactoringminer/astDiff/tests/TypeScriptDiffTest.java`.
  The warmed gate took `21.20s`; semantic remains opt-in and the next quality
  work should target graph/history/fusion for coupled source misses.
- Treat cloud embeddings/reranking as disabled unless an explicit repo policy
  allows them.

The release gate can run this proof optionally:

```bash
CTXHELM_BENCHMARK_CONFIG=/absolute/path/to/retrieval-quality.json bash scripts/release-gate.sh
```

When enabled, the gate fails if the proof cannot be generated, if the
proof/embedded benchmark privacy status is not local-only, if the v2.3 summary
is missing fixed corpus identity or paired baseline verdict fields, if
feature-export privacy regresses, if learned-policy status allows silent
defaults, if proof-boundary language is missing, if current reachable
retrieval-gap summaries are not resource-backed with context-area URIs and
next-read paths, if a benchmark repository report is missing, if a corpus has
insufficient evidence because history is unavailable, if the pinned broad fixed
corpus regresses below its recorded per-repository floors, if corpus verdicts
are missing source-recall fields or show source recall regression beyond the
release tolerance, or if `releaseGate.decision != "promote"`. A configured proof
where any required corpus only matches or trails lexical retrieval blocks
default promotion.

The deterministic release smoke is:

```bash
bash scripts/smoke-v23-eval.sh
```

It creates a small local git repository and checks source-free feature export,
feedback recording, offline learned policy, paired baselines, runtime
diagnostics, and product proof. RefactoringMiner and multi-repo suites remain
optional external gates:

```bash
ctxhelm eval benchmark --config .ctxhelm/benchmarks/refactoringminer-v23.json --format markdown
CTXHELM_BENCHMARK_CONFIG="$(pwd)/.ctxhelm/benchmarks/refactoringminer-v23.json" bash scripts/release-gate.sh
```

If the external checkout is not available, skip with the reason "external corpus unavailable." The skip is acceptable for the default release gate; it is
not evidence that the large-history proof passed.
