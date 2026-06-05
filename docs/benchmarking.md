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

Paired real-agent reports are a separate proof layer from historical retrieval
evals. They compare what ctxhelm surfaced with what the agent actually read.
`ctxhelmEvidenceTargetHits` means the plan or pack surfaced the target path,
`ctxhelmEvidenceOnlyTargets` means the path was surfaced but not consumed by a
native read, and `ctxhelmEvidenceMissedTargets` means the ctxhelm evidence did
not surface the target. This distinction prevents agent rate limits or
under-reading behavior from being misreported as retrieval quality.
Single-run comparisons and suite aggregates also include
`recommendedResearchActions`. These source-free actions route the next R&D step:
retry a real client after rate limits or client failures, collect real-client
evidence when only skipped contract reports exist, fix retrieval/query
construction when ctxhelm evidence misses targets, improve agent-consumption
guidance when surfaced targets are not read, harden required-call guidance when
observed ctxhelm calls are malformed or incomplete, or analyze the native
baseline when comparable lanes show no measured lift.

Experience-memory outcome suites are a paired real-agent specialization. The
`scripts/e2e-codex-memory-outcome-suite.sh` harness seeds approved source-free
experience cards from older repeated-file tasks, runs real read-only Codex CLI
comparisons on newer tasks, and reports source-free aggregate fields such as
`memoryTargetReadImprovedPairCount`,
`memoryTargetReadMatchedOrImprovedPairCount`,
`memoryIrrelevantReadImprovedPairCount`, evidence misses, under-read targets,
forbidden command counts, required ctxhelm-call compliance, and client
availability. Phase 253's three-repo run reports 3/3 memory target-read
improvements with no evidence or client failures, but zero irrelevant-read
improvements, so it is a target-consumption proof rather than an efficiency
proof. Phase 255 tightens the memory-lane prompt and reruns that same
three-repo suite; target-read improvement remains 3/3 and
`memoryIrrelevantReadImprovedPairCount` moves to 3/3, so the current measured
memory outcome guard covers both target consumption and read efficiency.

Historical eval and product-proof reports also expose
`recommendedResearchActions`. Historical reports route candidate gaps to
candidate generation, generated-but-unselected misses to ranking or budget
allocation, context-area recovery to progressive native-read guidance, broader
agent-evidence-only recovery to next-read alignment, memory reuse evidence to
experience-memory collection or selection work, validation gaps to test mapping,
and graph ablation/profile evidence to graph edge-budget work. Product proof
reports aggregate corpus verdicts into fixture/history refresh, runtime,
protected-evidence, retrieval/ranking regression, native-baseline gap, BM25
backend evidence, memory reuse proof, or preserve-contract actions. These
recommendations use only counts, labels, verdicts, and source-free reasons.

Historical eval reports include `memoryReuseSummary` to make experience-memory
reuse measurable instead of anecdotal. It reports how many evaluated commits had
memory candidates, how many memory candidates were selected within the top-10
context budget, how many retrieval targets memory hit or missed while memory was
active, and whether memory added unique target hits or unique non-targets beyond
the lexical baseline. The summary stores counts and role labels only.
`scripts/smoke-memory-history-lift.sh` exercises this path in a controlled
history: before approval, memory contributes no candidates; after approval, the
historical report records a memory-only target hit beyond lexical and routes the
next action to `evaluate_memory_reuse_lift`.
`scripts/smoke-memory-parent-snapshot-lift.sh` covers the non-root commit path:
historical eval builds a parent snapshot, projects source-free approved memory
from the source checkout into that snapshot's local store, and requires the
snapshot report to preserve a memory-only target hit beyond lexical.
`scripts/smoke-memory-benchmark-lift.sh` exercises the higher-level aggregation
path: two local repositories seed approved source-free experience cards, run
`eval proof`, and require product proof to preserve per-repo memory-only target
hits beyond lexical while routing product-level R&D to
`evaluate_memory_reuse_lift`. It proves benchmark/report plumbing, not broad
memory generalization on arbitrary histories.

`scripts/measure-memory-generalization.sh` is the broader real-corpus R&D
measurement path. It scans a local repository for repeated-file commit pairs,
evaluates each newer commit before and after approving an experience card seeded
from the older commit, and writes a source-free aggregate report with
`memoryUniqueLiftPairs`, `memoryUniqueTargetHitCount`,
`memoryUniqueNonTargetCount`, final-pack impact counters,
`precisionNeedsWork`, and `generalizationProven`. This harness should be used
before claiming memory has generalized beyond controlled smokes, because it
records both target lift and whether memory actually changed the context pack.
The pack-impact counters distinguish signal-level overlap from user-visible
selection changes:

- `memoryPackChangedPairs`: evaluated pairs where approving memory changed the
  final top-10 context pack.
- `memoryPackAddedTargetCount`: retrieval targets added to the final pack by
  memory.
- `memoryPackAddedNonTargetCount`: non-target files added to the final pack by
  memory.
- `memorySignalOnlyNonTargetCount`: memory non-target overlap that did not add a
  new final-pack file.

`precisionNeedsWork` is driven by pack-added non-target pressure, not by
signal-only overlap. Signal-only overlap is still reported through
`signalOnlyMemoryOverlapObserved` and routed to
`track_signal_only_memory_overlap` so it can be monitored without blocking
ranking work.

Phase 220 uses that harness as a precision regression check. On the same
two-pair RefactoringMiner slice, preserving experience-card recommendation order
and capping source-like memory context candidates kept `memoryUniqueLiftPairs =
1` and `combinedRecoveredPairs = 1`, while reducing
`memoryUniqueNonTargetCount` from `8` to `2`. The remaining report still sets
`precisionNeedsWork = true`, so broader multi-pair and multi-repo proof is still
required before stronger memory-generalization claims.

`scripts/measure-memory-generalization-suite.sh` runs the same measurement
across multiple local repositories and aggregates the source-free reports.
Phase 221 measured RefactoringMiner, VeriSchema, ReAgent, and ctxhelm with one
repeated-file pair per repo. The suite found `memoryUniqueLiftPairs = 2`,
`memoryUniqueTargetHitCount = 2`, `combinedRecoveredPairs = 1`, and
`memoryUniqueNonTargetCount = 3`, with `multiRepoMeasured = true` and
`generalizationProven = true` under the current bounded criterion. It also kept
`precisionNeedsWork = true`, so the next benchmark step is increasing
pairs-per-repo and comparing memory selection against graph and semantic
ablations.

Phase 234 separates memory signal pressure from final-pack impact after the
corroboration policy work. A six-repo semantic-enabled suite over 30 evaluated
pairs preserves `memoryUniqueLiftPairs = 2` and `memoryUniqueTargetHitCount = 2`
while reporting `memoryPackChangedPairs = 0`,
`memoryPackAddedNonTargetCount = 0`, `memorySignalOnlyNonTargetCount = 1`,
`unsupportedMemoryNoiseRepositoryCount = 0`, `precisionNeedsWork = false`, and
`generalizationProven = true`. The remaining memory non-target is therefore a
tracked signal-only overlap diagnostic, not a context-pack regression.

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
  global release threshold. Release promotion should be judged against the
  selected release/archive binary path, because debug `cargo run` cold proofs
  can overstate planner runtime for large fixtures without changing retrieval
  quality verdicts.
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

Phase 252 fixes the semantic gate classification around this case. Named
regressions from the eval-only `local_metadata_reranked` variant remain visible
in `namedRegressions`, but they no longer force the top-level semantic decision
to `block`. With the real local `local_fastembed` backend, the current
source-free semantic gate state is:

| Repo | Decision | Default Recall@K | Local Semantic Recall@K | Semantic Delta | Notes |
| --- | --- | ---: | ---: | ---: | --- |
| RefactoringMiner | `hold` | `0.48541665` | `0.5104166` | `+0.025` | Small lift, not enough for default promotion. |
| ctxhelm | `hold` | `0.3212704` | `0.3212704` | `+0.000` | Neutral recall. |

That is still not a default-promotion result. It is a more accurate result:
semantic is local, source-free, and useful for continued R&D, while default
ranking remains guarded until repeated query-family lift clears the gate.

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
- Phase 187 adds a source-history reserve for co-change source candidates only
  when another source-safe signal corroborates the path. The clean four-repo
  release proof still promotes; source recall improves on ctxhelm
  (`0.42857143 -> 0.5`) and VeriSchema (`0.32258064 -> 0.38709676`). The same
  proof records a ctxhelm all-file Recall@10 tradeoff (`0.6666667 -> 0.5587302`)
  from broad-doc displacement, so this should be read as a source-channel
  improvement rather than an all-file recall improvement.
- Phase 188 adds `selectedSignalProfiles` to historical commit eval reports.
  Each commit now reports selected top-10 counts by source-free signal family and
  file role, plus selected retrieval-target counts. The four-repo release proof
  still promotes with unchanged recall metrics, and all 16 evaluated commits
  include non-empty profiles. Use this field to debug budget tradeoffs such as
  co-change source slots versus broad governance docs before changing ranking
  allocation again.
- Phase 189 uses those profiles to rebalance broad target-file allocation. Broad
  root governance docs now run before broad source-history reserve, while
  source-history remains protected for standard tasks and for broad tasks without
  governance pressure. Source-history candidates now prefer module entrypoints
  such as `src/lib.rs`. The four-repo proof still promotes; ctxhelm File
  Recall@10 improves (`0.5587302 -> 0.67777777`) while ctxhelm Source Recall@10
  stays `0.55`.
- Phase 190 adds `coveragePercent` and `inspectionPressure` to plan-level
  `contextAreas` and renders those fields in context-pack guidance. This makes
  broad-area hints more actionable for progressive native reads without spending
  more top-10 target-file budget. The four-repo proof still promotes with Phase
  189 file/source/test/context-area metrics unchanged.
- Phase 191 adds `inspectionPressureBreakdown` to plan-level `contextAreas`,
  renders that source-like/validation/docs split in packs, and propagates it to
  retrieval-gap summaries when area profiles are available. The product-proof
  checker now validates emitted pressure arithmetic, and the four-repo proof
  still promotes with Phase 190 metrics unchanged.
- Phase 192 adds `contextAreaPressureSummary` to historical eval reports and
  product-proof embeddings. The summary aggregates total broad-area pressure,
  source-like/validation/docs pressure, zero-selected areas, and the
  highest-pressure area so broad-area bottlenecks are visible at repository
  scale without reading source. The four-repo proof still promotes with Phase
  191 metrics unchanged.
- Phase 193 adds `contextAreaNextReadSummary` to historical eval reports and
  product-proof embeddings. The summary counts top-10 misses recoverable from
  progressive context-area `nextReadPaths`, including top-pressure and
  zero-selected-area recovery. The four-repo proof still promotes with Phase
  192 metrics unchanged while showing ctxhelm and VeriSchema misses that are
  recoverable by native agent reads after the selected-file budget is exhausted.
- Phase 194 makes `nextReadPaths` ordering source-free and signal-aware. Within
  each context area, unselected paths now prefer source/config/schema roles and
  stronger local signals before weaker progressive reads. The four-repo proof
  still promotes with selected-file metrics unchanged while next-read recovery
  improves from `9 -> 10` on ctxhelm and `10 -> 14` on VeriSchema.
- Phase 195 makes `nextReadPaths` budgets adaptive for high-pressure
  source-like and validation-heavy context areas. Low-pressure areas stay at
  four paths, while high-pressure areas can expose six or eight bounded
  progressive reads. The four-repo proof still promotes with selected-file,
  source, test, validation, and broad-area metrics unchanged while next-read
  recovery improves from `10 -> 11` on ctxhelm and `14 -> 16` on VeriSchema.
- Phase 196 reserves selected validation areas in broad context-area guidance
  and adds package-mirrored related-test affinity. The accepted four-repo proof
  still promotes with selected-file, source, test, and validation metrics
  unchanged while VeriSchema broad context-area recall improves from
  `0.5777778 -> 0.84444445` and next-read recovery improves from `16 -> 19` of
  `39` missed@10 files.
- Phase 197 adds `agentEvidenceRecoverableCount` to
  `contextAreaNextReadSummary`. The existing `nextReadRecoverableCount` remains
  the progressive-read-only count, while the new field also counts selected
  related tests and selected context files. The four-repo proof still promotes
  with retrieval metrics unchanged and shows VeriSchema `29 / 39` missed@10
  files recoverable through the full agent evidence bundle.
- Phase 198 adds `candidateCoverageSummary` to historical eval reports and
  product-proof embeddings. The summary counts selected-file misses that were
  already generated as candidates but ranked outside top-10, plus misses with no
  candidate signal. The four-repo proof still promotes with Phase 197 retrieval
  metrics unchanged while showing ctxhelm `11 / 12`, RefactoringMiner `1 / 1`,
  ReAgent `0 / 0`, and VeriSchema `36 / 39` missed@10 files are
  candidate-recoverable. This keeps the next ranking work focused on selection
  pressure instead of guessing at candidate-generation gaps.
- Phase 199 extends `candidateCoverageSummary` with source-free pressure
  profiles: recoverable role counts, recoverable signal counts, no-candidate
  role counts, and top candidate-recoverable context areas. The four-repo proof
  still promotes with Phase 198 metrics unchanged. VeriSchema now shows
  recoverable pressure in `schema_agent/agents=7`, `tests/agents=6`, and
  `tests/evaluation=6`, with signal pressure led by `co_change=17`,
  `related_test=17`, and `dependency=12`.
- Phase 200 adds a bounded contextual README doc reserve for broad tasks. The
  accepted four-repo proof promotes and improves average File Recall@10 from
  `0.658268` to `0.7082679`, average file delta versus lexical from
  `+0.17873949` to `+0.22873944`, Agent Evidence Recall@10 from `0.76052284`
  to `0.81052285`, Context Recall@10 from `0.7638889` to `0.7708334`, and
  VeriSchema File Recall@10 from `0.35529414` to `0.55529416` without
  source/test/effective-validation/broad-area regression. A source-dependency
  ordering experiment was rejected because it displaced a true workflow-script
  hit and regressed VeriSchema File Recall@10.
- Phase 201 adds source-free agent-evidence-only gap profiles to
  `contextAreaNextReadSummary`. The four-repo proof promotes with Phase 200
  retrieval metrics unchanged while showing the residual gap between
  progressive next reads and the full agent evidence bundle is validation-only:
  VeriSchema has `10` agent-evidence-only missed@10 files, all tests,
  concentrated in `tests/agents=5`, `tests/evaluation=4`, and `tests/core=1`;
  RefactoringMiner has `1` agent-evidence-only test gap. A broad agent-source
  reserve, role-aware related-test next-read priority, and larger high-pressure
  next-read cap were measured and rejected because they produced no recovery
  movement.
- Phase 202 renders those source-free agent-evidence-only profiles in
  historical eval markdown reports. Reports now show agent-evidence-only
  recovery count, role counts, and top areas, so validation-only residual gaps
  are visible without opening raw JSON. This is a report-surface improvement;
  retrieval metrics remain governed by the Phase 201 proof.
- Phase 203 surfaces selected related tests as source-free validation evidence
  in generated packs. Packs now include a `Related test evidence` section with
  test path, context area, reason, confidence, and targeted command details, and
  explicitly explain that selected validation evidence may not be repeated in
  context-area next-read lists. This improves agent consumption of evidence
  already counted by product proof without changing target-file ranking.
- Phase 204 hardens paired real-agent reports with source-free forbidden-tool
  accounting. Lane metrics now include forbidden tool-call counts/calls, suite
  reports aggregate them, and `ctxhelm eval agent-run` renders the count. A
  hardened Claude Code `2.1.159` run on the Phase 203 validation-evidence task
  observed no forbidden calls but no ctxhelm lift: native baseline covered
  `2 / 3` targets, while `ctxhelm-plan` and `ctxhelm-brief` each covered
  `1 / 3`; outcome claim `ctxhelm_matched`.
- Phase 205 adds source-free agent consumption diagnostics to paired real-agent
  reports. The harness now separates target discovery from actual native target
  reads, reports target-read coverage deltas, discovered-only targets,
  missed-target counts, read-role counts, missed-target role counts, and a
  `ctxhelmUnderReadTargetsObserved` flag. This makes the Phase 204 under-read
  behavior measurable before changing pack prompts or ranking. A Claude Code
  `2.1.159` run on the Phase 205 harness task showed `ctxhelm-brief` matching
  native target-read coverage at `0.67` while reducing irrelevant reads from
  `3` to `2`; the `ctxhelm-plan` lane exposed the remaining risk by discovering
  one target without actually reading it.
- Phase 206 hardens product-facing consumption guidance. `prepare_task` MCP text
  now tells agents that discovering a path is not the same as consuming it,
  generated agent guidance repeats that rule, and packs include a source-free
  `Consumption guidance` section before target files and snippets. Two Claude
  Code `2.1.159` paired runs were intentionally treated as noisy outcome
  evidence: the first showed `ctxhelm-plan` improving target-read coverage from
  `0.33` to `0.67`, while the second kept `ctxhelm-plan` target-read coverage
  at `0.67` but had a failed `ctxhelm-brief` lane before ctxhelm calls. This
  phase improves the agent-consumption contract; it does not claim a stable
  brief-pack outcome lift yet.
- Phase 207 hardens paired real-agent comparability. The harness now records
  required ctxhelm calls, observed required calls, missing required calls,
  `ctxhelmCallCompliance`, `evaluationStatus`, and `evaluationEligible` per
  lane. `ctxhelm-plan` requires `prepare_task`; `ctxhelm-brief` requires both
  `prepare_task` and `get_pack`. Single-run and suite reports expose
  `comparisonEligible`, eligible-lane counts, comparable ctxhelm lane counts,
  and missing-required-call observations. Outcome claims now fall back to
  `insufficient_comparable_lanes` when a baseline plus ctxhelm-assisted lane was
  not actually comparable, preventing failed or no-call ctxhelm lanes from being
  interpreted as stable retrieval/pack weakness.
- Phase 208 adds source-free real-client failure classification after a Claude
  Code probe hit a session rate limit before any lane could run. Lane reports
  now expose `clientFailureKind`, `clientApiErrorStatus`, and
  `rateLimitObserved`, while comparisons and suite aggregates expose whether any
  client failures or rate limits were observed. This keeps client availability
  failures separate from ctxhelm retrieval, pack, and consumption behavior
  without storing raw client output.
- Phase 209 validates required ctxhelm call arguments before treating a lane as
  comparable. `prepare_task` is only a valid required call when the sanitized
  request proves the explicit repo and task were passed. `get_pack` is only valid
  when it carries the explicit repo, task, `budget = "brief"`,
  `format = "json"`, and `recordTrace = false`. Reports now expose
  `requiredCtxhelmCallSpecs`, `invalidRequiredCtxhelmCalls`, invalid-call counts,
  and `invalidRequiredCtxhelmCallsObserved`, so wrong-repo or malformed MCP calls
  are not counted as ctxhelm outcome evidence.
- Phase 225 expands paired real-agent outcome measurement to the full planned
  lane matrix: native baseline, `ctxhelm-plan`, `ctxhelm-brief`,
  `ctxhelm-standard`, and `ctxhelm-memory`. Standard and memory lanes require
  valid explicit-repo `prepare_task` plus `get_pack` with
  `budget = "standard"`, `format = "json"`, and `recordTrace = false`. The
  first local Claude Code `2.1.163` attempt hit rate limits in every lane, so
  the artifact records `insufficient_comparable_lanes` and
  `retry_real_client_when_available` instead of claiming outcome lift.
- Phase 235 adds a source-free Claude Code client preflight to that lane matrix.
  When preflight detects a client failure, live lane execution is short-circuited
  while ctxhelm evidence is still collected. Rate-limited lanes now report
  `ctxhelmCallCompliance = client_unavailable` instead of false missing required
  calls. The fresh Claude Code `2.1.163` artifact still observes API status
  `429`, but `missingRequiredCtxhelmCallsObserved = false`,
  `ctxhelmEvidenceMissesObserved = false`, and plan/brief/standard/memory lanes
  each surface both expected targets.
- Phase 254 refreshes that status with the current Claude Code `2.1.163`
  client and the four-task R&D breadth suite. All four current preflights still
  report rate limiting, so the artifact is `degraded` with zero
  comparison-eligible tasks and zero comparable ctxhelm lanes. This remains
  client-availability evidence, not retrieval-quality evidence.
- Phase 222 adds memory-vs-signal R&D measurement. The multi-repo memory
  generalization suite can run with `--semantic --semantic-provider local_hash`
  and reports source-free semantic selected-target pairs, graph-edge ablation
  target-hit loss, graph/semantic memory-corroboration upper bounds, and
  uncorroborated memory lower bounds. The first four-repo semantic probe
  measured semantic target selections on two repositories, one memory-unique
  lift, one unique non-target memory selection, and zero semantic ablation-lift
  pairs, so memory remains useful but not yet precise enough for an automatic
  promotion policy.
- Phase 223 tightens memory precision accounting. Ranking no longer attaches
  memory to lexical-expansion-only paths and only allows an uncorroborated
  memory-only rescue when no target file was otherwise selected. Historical
  reports now separate `memoryUniqueNonTargetCount` from
  `memoryUniqueNonTargetWithoutCurrentSupportCount`. The four-repo rerun keeps
  one lexical-baseline-relative memory non-target, but reports zero unsupported
  memory non-targets and zero unsupported memory-unique target hits. That means
  the remaining memory "noise" is supported by another current signal, while
  raw lexical-baseline-relative noise stays visible.
- Phase 224 expands memory-generalization measurement. The suite now defaults to
  three pairs per repo, prefers distinct target files before duplicate-path
  repeated pairs, and reports `candidatePairCount`,
  `candidateTargetFileCount`, `evaluatedTargetFileCount`,
  `largerPairCountMeasured`, and `pairDiversityMeasured`. The four-repo
  semantic-enabled fixture run measured 12 pairs across 12 distinct target
  files, restored `memoryUniqueLiftPairs = 2`, and kept unsupported pure-memory
  noise at zero. The next promotion bar is paired real-agent outcome lift.
- Phase 228 tightens memory-noise routing. Memory-generalization reports now
  expose `memoryUniqueNonTargetWithCurrentSupportCount` and
  `supportedMemoryNoiseNeedsReview`, so raw lexical-baseline-relative non-targets
  supported by current signals route to signal-pressure review instead of
  uncorroborated-memory demotion. The fresh four-repo semantic-enabled suite
  measured `memoryUniqueNonTargetCount = 4`,
  `memoryUniqueNonTargetWithCurrentSupportCount = 4`, and
  `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`.
- Phase 229 makes that signal-pressure review measurable. Historical memory
  summaries and the single/suite harnesses now expose
  `memoryUniqueNonTargetCurrentSupportSignalCounts`,
  `memoryUniqueTargetHitCurrentSupportSignalCounts`, and
  `supportedMemoryNoiseDominantSignals`. The fresh four-repo semantic-enabled
  suite keeps `memoryUniqueLiftPairs = 2` and zero unsupported memory
  non-targets, while showing supported memory non-target pressure dominated by
  `dependency`, `lexical_expansion`, and `symbol`. The next ranking experiment is
  therefore `tune_memory_weight_against_supported_signal_pressure`, not a blind
  memory demotion.
- Phase 230 runs that weighting experiment. Memory now attaches to existing
  candidates only when they have strong current support: anchor, current diff,
  lexical, semantic, or co-change. Dependency-only and symbol-only candidates
  keep their own ranking evidence but no longer receive extra memory pressure.
  The fresh four-repo semantic-enabled suite keeps `memoryUniqueLiftPairs = 2`,
  keeps `memoryUniqueNonTargetWithoutCurrentSupportCount = 0`, and reduces
  `memoryUniqueNonTargetCount` from `4` to `1`. The rerun reports
  `weakSupportedMemoryNoiseNeedsTuning = false`, so the remaining local memory
  work is strong-signal overlap inspection and larger pair-count validation.
- Phase 231 runs that larger pair-count validation. The suite now reports
  `largerPairValidationTargetMet`, and the four-repo semantic-enabled run
  evaluates 20 pairs across 20 distinct target files. Memory lift remains
  `memoryUniqueLiftPairs = 2`, unsupported memory non-targets remain zero, and
  weak-supported memory noise remains cleared. The remaining two memory
  non-targets are both supported by strong current signals and localized to
  VeriSchema, so the next local R&D route is repository diversity plus
  strong-signal overlap inspection, not more same-corpus pair-count expansion.
- Phase 232 adds an explicit repository-diversity target to the suite. Reports
  now expose `repositoryDiversityTarget`, `repositoryDiversityTargetMet`,
  `repositoryDiversityNeedsExpansion`, `memoryLiftRepositoryCount`,
  `memoryNonTargetRepositoryCount`, `unsupportedMemoryNoiseRepositoryCount`, and
  `strongSupportedMemoryNoiseRepositoryCount`. The six-repo semantic-enabled
  run over VeriSchema, ReAgent, ctxhelm, flask, fd, and express evaluates 30
  pairs across 30 distinct target files and sets
  `repositoryDiversityTargetMet = true`. It preserves `memoryUniqueLiftPairs =
  2`, but finds one unsupported memory non-target in express, so the next local
  memory R&D route is `demote_uncorroborated_memory_candidates` and
  `test_memory_candidate_corroboration_policy`, not further diversity expansion.
- Phase 233 tightens uncorroborated memory rescue. Ranking no longer adds a
  memory-only source file when native related-test evidence is already
  available, but still preserves memory-only rescue when memory is the only
  available evidence. The express focused rerun drops
  `memoryUniqueNonTargetWithoutCurrentSupportCount` from `1` to `0`. The
  six-repo semantic-enabled rerun preserves `memoryUniqueLiftPairs = 2`, lowers
  `memoryUniqueNonTargetCount` from `2` to `1`, sets
  `unsupportedMemoryNoiseRepositoryCount = 0`, and routes remaining local memory
  work to strong-signal overlap inspection.
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
release tolerance, if context-area pressure or next-read recovery summaries
emit inconsistent arithmetic or source text, or if `releaseGate.decision !=
"promote"`. A configured proof
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
