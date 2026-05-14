# Retrieval Benchmarking

ctxpack v1.2 uses source-free benchmark suites to answer the product question:

> Does ctxpack help an agent choose better files and tests than native search alone?

The benchmark runner reuses `ctxpack eval history` for each configured repository. It records file/test recall, lexical baseline comparison, signal ablations, grouped retrieval gaps, skipped path counts, and privacy status without storing source snippets, prompt text, or commit subjects.

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
```

The Markdown report is meant for local inspection. The JSON report is the stable contract for future release gates, portfolio tables, and regression comparison.

## Privacy Boundary

Benchmark reports include:

- suite and repository labels;
- repo IDs;
- revision range metadata;
- recall and baseline metrics;
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
