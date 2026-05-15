# Local Semantic Retrieval

ctxpack supports optional local semantic retrieval as a secondary signal in the context compiler. It is disabled by default, uses the same safe inventory policy as lexical search and packs, and does not call cloud embedding or reranking services.

## Enable Per Invocation

Semantic retrieval is explicit on each workflow that can use it:

```bash
ctxpack search "payment webhook validation" --repo /path/to/repo --semantic
ctxpack prepare-task "fix payment webhook validation" --repo /path/to/repo --semantic
ctxpack get-pack "fix payment webhook validation" --repo /path/to/repo --semantic
ctxpack eval history --repo /path/to/repo --semantic
```

The current provider is `local_hash` with model `ctxpack-local-hash-v1`, cosine similarity, and local vector metadata only. It is a deterministic local provider meant to prove the semantic-retrieval contract without introducing cloud defaults.

## Index Vector Metadata

To persist source-free vector metadata in the local SQLite store:

```bash
ctxpack index --repo /path/to/repo --semantic
ctxpack storage status --repo /path/to/repo
```

`--semantic` implies a safe inventory storage sync. The store records provider, model, dimensions, distance metric, file path, safe hash, privacy label, and numeric vector JSON. It does not store raw file contents, prompts, snippets, secrets, or cloud payloads.

## Agent And MCP Use

MCP `prepare_task`, `get_pack`, and `search` accept `semantic: true`. The field is optional and additive, so existing agents keep their lexical, symbol, graph, test, and history behavior unless they explicitly request semantic retrieval.

Semantic evidence appears as the `semantic` retrieval signal with provider metadata in source-free provenance. It is intentionally weighted below exact path, active diff, symbol, lexical, graph, and test evidence so conceptual matches cannot crowd out stronger code signals.

## Privacy Boundary

Semantic retrieval uses:

- `.gitignore`, `.ctxpackignore`, and `.cursorignore`
- generated-file and sensitive-file exclusions
- source-read revalidation before vectorization
- local-only privacy status
- no cloud embedding or reranking calls

Cloud embeddings and cloud reranking remain out of scope for the default product.

## When To Avoid It

Avoid semantic retrieval for exact identifier, stack trace, route, config-key, or single-file edit tasks where lexical or explicit path anchors are stronger. Use it when the task is conceptual, such as finding payment webhook validation, retry logic, normalization flows, or analogous feature patterns.

## Validation

Maintainers can run deterministic local coverage with:

```bash
bash scripts/smoke-semantic.sh
```

The release gate also runs `scripts/smoke-semantic.sh` and checks that source and secret sentinels are not persisted in ctxpack local state.
