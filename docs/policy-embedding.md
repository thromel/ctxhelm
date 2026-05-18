# Policy And Embedding Controls

ctxpack keeps semantic retrieval and cloud model use explicit. Default context
planning remains local lexical, graph, tests, history, memory, and feedback
signals. Local semantic retrieval is opt-in, and cloud embeddings/reranking stay
disabled unless a future policy gate explicitly enables them.

Inspect the local semantic provider:

```bash
ctxpack semantic status --repo /path/to/repo --query "payment webhook validation"
```

The current provider is `local_hash` with model ID `ctxpack-local-hash-v1`.
It is deterministic, source-free in reports, and exists to make semantic
policy paths testable without enabling cloud embeddings or cloud reranking.

Compare retrieval-policy experiment rows without changing default ranking:

```bash
ctxpack eval policy experiments "fix requireSession bug" --repo /path/to/repo
```

The policy experiment report includes lexical-only, hybrid local semantic,
graph neighborhood, and current default rows. It is report-only and includes
`sourceTextLogged: false`.

## Source Boundary

These reports are metadata-only. They include provider kind, model ID,
dimensions, vector counts, semantic usage counts, graph node/edge counts,
recall metrics, diagnostics, and explicit cloud-disabled flags. They do not
include raw source text, prompts, terminal logs, model transcripts, cloud
payloads, or vector-provider request bodies.

## Smoke Test

Maintainers can run:

```bash
bash scripts/smoke-policy-embedding.sh
```

The smoke verifies local semantic provider status, cloud-disabled policy flags,
policy experiment rows, and absence of source sentinel leakage.
