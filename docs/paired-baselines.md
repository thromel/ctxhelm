# Paired Baselines And Ablations

`ctxhelm eval baselines` compares the default ctxhelm context-file ranking
against simpler baselines and single-signal variants on the same historical
commit corpus. The report is source-free: it records paths, metrics, signal
families, verdicts, and privacy status, but not source snippets, prompt text,
commit subjects, stack traces, or terminal logs.

## Run

```bash
ctxhelm eval baselines --repo /path/to/repo --limit 20 --budget 10
ctxhelm eval baselines --repo /path/to/repo --limit 20 --budget 10 --format json
ctxhelm eval baselines --repo /path/to/repo --cache --parallelism 4 --format markdown
```

Use `--min-lift` and `--max-regression` to tune verdict thresholds:

```bash
ctxhelm eval baselines --repo /path/to/repo --min-lift 0.05 --max-regression 0.02
```

## Compared Variants

The report always includes:

- `ctxhelm_default`: the current hybrid ranking under the fixed context-file
  budget.
- `lexical_baseline`: exact/BM25-style path and identifier ranking.
- `no_context`: zero-file baseline for an agent starting without ctxhelm
  context.
- `semantic_only`: candidates that carried the semantic signal.
- `graph_only`: candidates that carried dependency/graph signal.
- `history_only`: candidates that carried co-change/history signal.
- `test_only`: candidates that carried related-test signal.
- `memory_only`: candidates that carried durable memory-card signal.
- `feedback_weighted`: currently marked `insufficient_evidence` until feedback
  labels are joined with fixed-corpus candidate rows.
- `without_*`: ablation rows from the historical eval signal-ablation engine.

Signal-only variants filter the same source-free candidate set used by the
default ranking. They are diagnostic controls, not a replacement for an
independently trained ranker.

## Metrics

Each row includes:

- Recall@K over safe changed files.
- Precision@K proxy over the fixed context-file budget.
- MRR@K.
- Test recommendation rate.
- Average recommended context files.
- Delta against `ctxhelm_default`.
- Delta against `lexical_baseline`.
- Verdict.

The report also carries:

- token ROI rows for brief, standard, and deep budget presets;
- validation coverage from related-test recommendation rate;
- signal saturation rows, showing how often each signal appeared and how much
  recall it delivered by itself;
- retrieval gap taxonomy from grouped missed-file families;
- runtime diagnostics and local-only privacy status.

## Verdicts

Verdicts are thresholded so raw deltas do not get over-read:

- `lift`: the variant beats lexical by more than `--min-lift`.
- `neutral`: the variant is within configured thresholds.
- `regression`: the variant falls below default by more than
  `--max-regression`.
- `insufficient_evidence`: the corpus or signal had no usable evidence.

`lexicalStatus` and `lexicalDeltaAtK` call out whether default ctxhelm is ahead
of, tied with, or behind exact lexical search. Lexical parity is not failure:
it may still mean ctxhelm adds tests, validation commands, graph evidence, and
progressive context structure. Lexical regression is serious and should block
world-class claims until the missed-family taxonomy explains it.

## Lexical Backend Comparison

`ctxhelm eval baselines` treats the active lexical implementation as the
lexical baseline. After the BM25 backend landed, use `ctxhelm eval lexical
compare` when you need to compare that active backend against the previous
heuristic scanner on a single source-free query:

```bash
ctxhelm eval lexical compare --repo /path/to/repo --query "requireSession" --limit 10
ctxhelm eval lexical compare --repo /path/to/repo --query "requireSession" --limit 10 --format json
```

The comparison report includes:

- query hash, not raw query text;
- source-free result rows containing path, role, language, and score;
- overlap@limit, top-path changes, BM25-only paths, and legacy-only paths;
- backend diagnostics summarized by code/severity count;
- local-only privacy status with source text and result reasons omitted.

Use this command for R&D slices that need to measure whether fielded BM25 and
symbol facets changed candidate ordering before promoting broader retrieval
claims.

## Privacy Boundary

The JSON output is intended for local eval storage, benchmark comparison, and
future policy learning. It must stay source-free. Treat a report as invalid if
it includes raw source text, commit subjects, user prompts, stack traces,
terminal output, or secret-bearing values.
