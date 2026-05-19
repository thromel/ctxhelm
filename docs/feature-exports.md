# Candidate Feature Exports

Candidate feature exports are source-free rows for learning, diagnostics, paired
analysis, and future offline retrieval-policy experiments.

They answer:

```text
Which retrieval candidates did ctxpack consider, what signals selected them,
and which safe labels are available for evaluation?
```

They do not store source snippets, prompt text, issue descriptions, terminal
logs, stack traces, commit subjects, or secret-bearing values.

## Export

```bash
ctxpack eval features export "fix login redirect" --repo /path/to/repo --format markdown
ctxpack eval features export "fix login redirect" --repo /path/to/repo --format json
ctxpack eval features export "fix login redirect" --repo /path/to/repo --semantic --limit 200
```

By default, the export is written under local ctxpack state:

```text
~/.ctxpack/repos/{repo_id}/feature-exports/{export_id}.json
```

Use `--no-store` to print the export without writing it.

## Manage

```bash
ctxpack eval features list --repo /path/to/repo
ctxpack eval features inspect <export_id> --repo /path/to/repo --format json
ctxpack eval features compare --base-export <id> --head-export <id> --repo /path/to/repo
ctxpack eval features delete <export_id> --repo /path/to/repo --yes
```

Delete is a dry run unless `--yes` is passed.

## Row Shape

Each row includes:

- candidate kind, path, role, rank, selected rank, confidence, and reason code;
- signal scores plus normalized lexical, semantic, graph, history, test, memory,
  and feedback score fields;
- graph distance, history commit count, test relation confidence, memory count,
  and feedback event count where available;
- source-free labels such as `selected` or `unknown`;
- `sourceTextLogged: false`.

The first implementation exports plan candidates. Historical gold labels,
feedback read/edit labels, and paired baseline feature rows build on this
contract in later v2.3 phases.
