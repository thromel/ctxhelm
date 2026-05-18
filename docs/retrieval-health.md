# Retrieval Health

Retrieval health reports summarize whether ctxpack is selecting useful context
over time. They combine source-free historical eval evidence with source-free
feedback policy evidence.

Run:

```bash
ctxpack eval health --repo /path/to/repo --limit 20 --format markdown
ctxpack eval health --repo /path/to/repo --limit 20 --format json
```

The report includes:

- file and test recall from historical eval
- ctxpack lift over lexical/no-context baselines
- token ROI by pack budget
- feedback-derived context precision and validation coverage
- signal contribution summaries
- repeated retrieval gap families
- low-confidence flags
- `sourceTextLogged: false`

## Source Boundary

Retrieval health reports are metadata-only. They do not include raw source text,
raw prompts, terminal logs, or model transcripts. Paths, counts, metric names,
reason families, confidence values, and privacy flags are allowed.

The contract explicitly does not include raw source text.

## Smoke Test

Maintainers can run:

```bash
bash scripts/smoke-retrieval-health.sh
```

The smoke creates a temporary repository with real git history, runs
`ctxpack eval health`, verifies JSON and Markdown output, and rejects a source
sentinel from the report artifacts.
