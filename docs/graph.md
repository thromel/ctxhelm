# Graph Neighborhoods

Graph neighborhoods expose source-free relationships around a task or explicit
path anchors.

Run:

```bash
ctxpack graph neighborhood "fix requireSession bug" \
  --repo /path/to/repo \
  --mode bug-fix \
  --path src/auth/session.ts \
  --format markdown
```

The report includes:

- anchor paths
- file and test nodes
- dependency, test, memory, and feedback edges where available
- module/community summaries
- cap diagnostics when `--max-nodes` or `--max-edges` truncates output
- `sourceTextLogged: false`

Graph expansion is non-recursive by default. The report expands around anchors
only and caps output with explicit limits.

## Source Boundary

Graph reports are metadata-only. They do not include raw source text, prompts,
terminal logs, or model transcripts. They include paths, roles, edge labels,
confidence weights, reason codes, counts, diagnostics, and privacy flags.

## Smoke Test

Maintainers can run:

```bash
bash scripts/smoke-graph.sh
```

The smoke creates a temporary repository with imports and tests, exports graph
JSON and Markdown, checks source-free node/edge/community fields, and rejects a
source sentinel from the report artifacts.
