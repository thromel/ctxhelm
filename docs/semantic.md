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

The default provider is `local_hash` with model `ctxpack-local-hash-v1`, cosine similarity, and local vector metadata only. `local_hash` is deterministic scaffold/test behavior. It exists to prove the semantic-retrieval contract, storage privacy boundary, and agent provenance without claiming production retrieval quality.

## Source-Free Semantic Documents

Semantic retrieval now indexes source-free semantic documents instead of raw file bodies. A semantic document is built from safe repository metadata:

- path, role, language, and safe file hash
- extracted symbol names, kinds, line ranges, and sanitized signatures
- import/dependency edges
- related-test paths and source-free relation reasons
- docs/card file references
- optional precision edge labels from `.ctxpack/precision-edges.json`

The document report is visible through semantic status:

```bash
ctxpack semantic status --repo /path/to/repo --format json
```

The report includes `semanticDocumentCount`, `semanticFacetCount`, `precisionStatus`, `sourceTextLogged`, and `privacyStatus`. `sourceTextLogged` remains `false`; raw source bodies are not embedded, exported, or persisted by the default semantic document path.

Phase 56 also defines `local_fastembed` as the optional real local embedding backend. It is compiled only when `ctxpack-index` is built with the `local-embeddings` Cargo feature. Normal workspace builds keep `local_fastembed` unavailable and return the warning diagnostic `semantic_provider_unavailable` if it is requested, rather than falling back silently or calling a cloud provider.

## Index Vector Metadata

To persist source-free vector metadata in the local SQLite store:

```bash
ctxpack index --repo /path/to/repo --semantic
ctxpack storage status --repo /path/to/repo
```

`--semantic` implies a safe inventory storage sync. The store records provider, model, dimensions, distance metric, file path, safe hash, privacy label, source-free semantic document metadata, and numeric vector JSON for providers such as `local_hash` and `local_fastembed`. It does not store raw file contents, prompts, snippets, secrets, or cloud payloads.

## Agent And MCP Use

MCP `prepare_task`, `get_pack`, and `search` accept `semantic: true`. The field is optional and additive, so existing agents keep their lexical, symbol, graph, test, and history behavior unless they explicitly request semantic retrieval.

Semantic evidence appears as the `semantic` retrieval signal with provider metadata in source-free provenance. Search results include a `documentId`, bounded `matchedFacets`, and precision status so agents can tell whether a semantic match came from symbol, dependency, doc, related-test, or precision evidence. Provider reports expose `providerRole`, `qualityBackend`, `localOnly`, `providerAvailable`, `providerStatus`, `cacheLocation`, `semanticDocumentCount`, `semanticFacetCount`, `precisionStatus`, and `degraded` so agents and release checks can distinguish deterministic scaffold behavior from a production local backend.

Semantic retrieval is intentionally weighted below exact path, active diff, symbol, lexical, graph, and test evidence so conceptual matches cannot crowd out stronger code signals.

## Privacy Boundary

Semantic retrieval uses:

- `.gitignore`, `.ctxpackignore`, and `.cursorignore`
- generated-file and sensitive-file exclusions
- semantic-document construction before vectorization
- local-only privacy status
- no cloud embedding or reranking calls

Cloud embeddings and cloud reranking remain out of scope for the default product.
Semantic status and plans now include a `providerPolicy` object that records
allowed, denied, unavailable, disabled, and skipped backend decisions. Local
source-free metadata providers are allowed by default; cloud embeddings, cloud
reranking, source transfer, and reranking execution are denied or disabled until
repo policy explicitly opts in.

Promotion to a default retrieval signal is controlled by `ctxpack eval gate`.
The gate compares default retrieval against local semantic, precision-enriched
semantic, full hybrid, and policy-allowed reranked variants on a fixed corpus.
Neutral results produce `hold`; regressions or unsafe policy produce `block`.

Semantic defaults are not promoted in Phase 56. Promotion of any production-quality semantic backend remains gated by later evaluation and release criteria.

## When To Avoid It

Avoid semantic retrieval for exact identifier, stack trace, route, config-key, or single-file edit tasks where lexical or explicit path anchors are stronger. Use it when the task is conceptual, such as finding payment webhook validation, retry logic, normalization flows, or analogous feature patterns.

## Validation

Maintainers can run deterministic local coverage with:

```bash
bash scripts/smoke-semantic.sh
```

The release gate also runs `scripts/smoke-semantic.sh` and checks that source and secret sentinels are not persisted in ctxpack local state.
