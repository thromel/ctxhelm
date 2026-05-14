# Retrieval Benchmarking

ctxpack v1.2 uses source-free benchmark suites to answer the product question:

> Does ctxpack help an agent choose better files and tests than native search alone?

The benchmark runner reuses `ctxpack eval history` for each configured repository. It records file/test recall, lexical and no-context baseline comparison, signal ablations, token ROI, grouped retrieval gaps, skipped path counts, and privacy status without storing source snippets, prompt text, or commit subjects.

## Suite File

Benchmark suites are JSON files. Paths may be absolute or relative to the suite file.

```json
{
  "name": "retrieval-quality-smoke",
  "description": "Bounded source-free retrieval benchmark over local repos",
  "defaults": {
    "limit": 20,
    "rankingBudget": 10,
    "mode": "bug_fix",
    "targetAgent": "codex",
    "roleFilters": ["source", "test"]
  },
  "repositories": [
    {
      "name": "ctxpack",
      "path": ".",
      "base": "main~20",
      "head": "main"
    },
    {
      "name": "RefactoringMiner",
      "path": "../RefactoringMiner",
      "limit": 30,
      "rankingBudget": 10
    }
  ]
}
```

Fields:

- `name`: source-free suite label used in reports.
- `description`: optional source-free human context.
- `defaults.limit`: max historical commits per repository.
- `defaults.rankingBudget`: fixed context-file budget K used for combined, lexical, and ablation metrics.
- `defaults.mode`: task type used when replaying commit titles.
- `defaults.targetAgent`: source-free agent label in eval metadata.
- `defaults.roleFilters`: documented target roles for this benchmark scope. Phase 9 records these filters in source-free metadata; later v1.2 phases use them for deeper metric segmentation.
- `repositories[*].name`: source-free repo label.
- `repositories[*].path`: local repository path.
- `repositories[*].base` / `head`: optional stable revision range.
- `repositories[*].limit`, `rankingBudget`, `mode`, `targetAgent`, `roleFilters`: per-repo overrides.

## Run

```bash
ctxpack eval benchmark --config .ctxpack/benchmarks/retrieval-quality.json --format markdown
ctxpack eval benchmark --config .ctxpack/benchmarks/retrieval-quality.json --format json
ctxpack eval compare --base-report previous.json --head-report current.json --threshold fileRecallAt10=0.05
ctxpack eval proof --config .ctxpack/benchmarks/retrieval-quality.json
```

The Markdown report is meant for local inspection. The JSON report is the stable contract for future release gates, portfolio tables, and regression comparison. `ctxpack eval compare` consumes two benchmark JSON reports and emits source-free metric deltas, retrieval-gap family deltas, and threshold pass/fail checks. `ctxpack eval proof` runs the configured benchmark suite and emits the adoption-facing proof report with headline metrics, limitations, when ctxpack helps, when it does not, and future work from measured gaps.

## Privacy Boundary

Benchmark reports include:

- suite and repository labels;
- repo IDs;
- revision range metadata;
- recall and baseline metrics;
- no-context baseline metrics under the same fixed K budget;
- token ROI rows for brief, standard, and deep pack options;
- source-free file paths and role labels;
- skipped path counts and privacy status.

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

## Interpreting Metrics

- `rankingComparison.combined`: ctxpack's hybrid context-file ranking at the configured K budget.
- `rankingComparison.lexicalBaseline`: exact/BM25-style local search baseline under the same K budget.
- `rankingComparison.noContextBaseline`: a zero-file baseline that represents an agent starting without ctxpack-provided repository context.
- `signalAblations`: source-free lift checks after removing one retrieval signal family.
- `tokenRoi`: brief, standard, and deep budget estimates showing useful changed-file targets per 1k estimated context tokens.
- `largerPackAddsLittleValue`: true when a larger budget adds no additional useful changed-file targets over the previous budget in the current fixed ranking.
- `retrievalGapSummaries`: source-free failure families grouped by role, signal gap, package, path family, target status, and recommendation area.
- `ctxpack eval compare`: compares two benchmark reports across recall, token ROI, skipped paths, and gap families; configured thresholds fail when a metric drops more than the allowed amount.
- `ctxpack eval proof`: generates a concise product proof report from a local suite and embeds the underlying source-free benchmark report in JSON output.

## Product Proof

The product proof is intentionally narrow. It does not claim universal agent improvement. It answers:

- what local benchmark suite was run;
- how many repositories and commits were evaluated;
- headline recall, baseline lift, test recall, and token ROI metrics;
- when ctxpack helps;
- when ctxpack does not help;
- limitations in the benchmark design;
- which future milestone should address repeated gap families.

The release gate can run this proof optionally:

```bash
CTXPACK_BENCHMARK_CONFIG=/absolute/path/to/retrieval-quality.json bash scripts/release-gate.sh
```

When enabled, the gate fails if the proof cannot be generated or if the proof/embedded benchmark privacy status is not local-only.
