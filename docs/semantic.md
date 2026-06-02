# Local Semantic Retrieval

ctxhelm supports optional local semantic retrieval as a secondary signal in the context compiler. It is disabled by default, uses the same safe inventory policy as lexical search and packs, and does not call cloud embedding or reranking services.

## Enable Per Invocation

Semantic retrieval is explicit on each workflow that can use it:

```bash
ctxhelm search "payment webhook validation" --repo /path/to/repo --semantic
ctxhelm prepare-task "fix payment webhook validation" --repo /path/to/repo --semantic
ctxhelm get-pack "fix payment webhook validation" --repo /path/to/repo --semantic
ctxhelm eval history --repo /path/to/repo --semantic
```

The default provider is `local_hash` with model `ctxhelm-local-hash-v1`, cosine similarity, and local vector metadata only. `local_hash` is deterministic scaffold/test behavior. It exists to prove the semantic-retrieval contract, storage privacy boundary, and agent provenance without claiming production retrieval quality.

To request the production local embedding backend, pass the provider explicitly:

```bash
ctxhelm semantic status --repo /path/to/repo \
  --semantic-provider local_fastembed \
  --format json

ctxhelm index --repo /path/to/repo \
  --semantic \
  --semantic-provider local_fastembed

ctxhelm prepare-task "find payment webhook validation" \
  --repo /path/to/repo \
  --semantic \
  --semantic-provider local_fastembed

ctxhelm eval history --repo /path/to/repo \
  --semantic \
  --semantic-provider local_fastembed
```

`local_fastembed` remains local-only and source-free. It requires a binary built
with the `local-embeddings` Cargo feature; otherwise status and plans report the
provider as unavailable instead of falling back silently or calling a cloud
provider.

```bash
cargo build -p ctxhelm --features local-embeddings
```

## Production Local Embedding Status

Phase 62 measured `local_fastembed` on the same two-repo corpus as the v2.5
baseline. The production-local provider is source-free and correctly reports its
provider role, model, dimensions, and cache location, but it is not promoted as a
default retrieval signal because it did not improve Recall@10 and was materially
slower than default retrieval.

Measured Phase 62 status:

- `local_hash`: deterministic scaffold, model `ctxhelm-local-hash-v1`, 64 dimensions, not a quality backend.
- `local_fastembed`: production-local backend, quality backend `true`, local-only, no cloud source transfer.
- Default production-local model: `AllMiniLML6V2Q` with 384 dimensions.
- Jina code model status: still available explicitly as `JinaEmbeddingsV2BaseCode` with 768 dimensions, but too slow for the full two-repo eval loop in this implementation.
- Model cache: defaults to repo `.ctxhelm/cache/fastembed` when run inside a git repo, otherwise `CTXHELM_HOME/cache/fastembed`; override with `CTXHELM_FASTEMBED_CACHE_DIR`.
- Query-time vector cache: bounded in-process cache for repeated source-free document embeddings.
- Persisted query-vector cache: source-free query hashes and vectors are stored in SQLite so repeated fresh CLI/MCP processes can reuse query embeddings without storing raw query text.
- Expensive model prefilter: `local_fastembed` embeds at most 128 source-free candidate documents per query by default; override with `CTXHELM_FASTEMBED_DOCUMENT_LIMIT`.

Phase 62 result:

| Variant | RefactoringMiner Recall@10 | ctxhelm Recall@10 | Repo Runtime |
| --- | ---: | ---: | ---: |
| Default, semantic off | 0.7767 | 0.2299 | 26.3s |
| `local_hash` | 0.7767 | 0.2299 | 57.8s |
| `local_fastembed` `AllMiniLML6V2Q` | 0.7767 | 0.2299 | 183.7s |

Decision: keep `local_fastembed` opt-in for experiments and conceptual queries.
Do not promote it by default until a later retrieval path proves recall lift with
acceptable runtime.

## Source-Free Semantic Documents

Semantic retrieval now indexes source-free semantic documents instead of raw file bodies. A semantic document is built from safe repository metadata:

- path, role, language, and safe file hash
- Tree-sitter extracted symbol names, kinds, line ranges, and sanitized signatures
- import/dependency edges
- related-test paths and source-free relation reasons
- docs/card file references
- optional discovered or imported precision edge labels from `.ctxhelm/precision-edges.json`

The document report is visible through semantic status:

```bash
ctxhelm semantic status --repo /path/to/repo --format json
```

The report includes `semanticDocumentCount`, `semanticFacetCount`, `precisionStatus`, `sourceTextLogged`, and `privacyStatus`. `sourceTextLogged` remains `false`; raw source bodies are not embedded, exported, or persisted by the default semantic document path.

Phase 56 also defines `local_fastembed` as the optional real local embedding backend. It is compiled only when the `ctxhelm` binary, `ctxhelm-mcp`, `ctxhelm-compiler`, or `ctxhelm-index` is built with the `local-embeddings` Cargo feature. Normal workspace builds keep `local_fastembed` unavailable and return the warning diagnostic `semantic_provider_unavailable` if it is requested, rather than falling back silently or calling a cloud provider.

## Index Vector Metadata

To persist source-free vector metadata in the local SQLite store:

```bash
ctxhelm index --repo /path/to/repo --semantic
ctxhelm index --repo /path/to/repo --semantic --semantic-provider local_fastembed
ctxhelm index --repo /path/to/repo --semantic --semantic-provider local_fastembed --semantic-limit 128
ctxhelm storage status --repo /path/to/repo
```

`--semantic` implies a safe inventory storage sync. The store records provider, model, dimensions, distance metric, file path, safe hash, privacy label, source-free semantic document metadata, and numeric vector JSON for providers such as `local_hash` and `local_fastembed`. It does not store raw file contents, prompts, snippets, secrets, or cloud payloads.

For `local_fastembed`, the foreground default is intentionally bounded to a
small 16-document seed. Use `--semantic-limit` when you intentionally want a
larger local vector job, and prefer doing that outside an interactive agent turn
on large repositories. Semantic search reuses matching persisted document
vectors by exact path, safe hash, provider, model, dimensions, and distance
metric, then embeds only a missing query vector and candidate misses. Embedded
candidate misses are written through to the local source-free vector store, and
embedded queries are stored by query hash rather than raw query text, so
repeated fresh CLI or MCP processes can reuse both document vectors and query
vectors. If the file hash changes, the stored row is updated under the
path/provider/model identity and stale hashes are not reused. Provider or model
failures during indexing are reported as errors; ctxhelm no longer treats a
failed semantic vector build as a successful zero-vector index.

## Agent And MCP Use

MCP `prepare_task`, `get_pack`, and `search` accept `semantic: true`. The field is optional and additive, so existing agents keep their lexical, symbol, graph, test, and history behavior unless they explicitly request semantic retrieval.

Semantic evidence appears as the `semantic` retrieval signal with provider metadata in source-free provenance. Search results include a `documentId`, bounded `matchedFacets`, and precision status so agents can tell whether a semantic match came from symbol, dependency, doc, related-test, or precision evidence. Provider reports expose `providerRole`, `qualityBackend`, `localOnly`, `providerAvailable`, `providerStatus`, `cacheLocation`, `semanticDocumentCount`, `semanticFacetCount`, `precisionStatus`, and `degraded` so agents and release checks can distinguish deterministic scaffold behavior from a production local backend.

Semantic retrieval is intentionally weighted below exact path, active diff, symbol, lexical, graph, and test evidence so conceptual matches cannot crowd out stronger code signals.

## Privacy Boundary

Semantic retrieval uses:

- `.gitignore`, `.ctxhelmignore`, and `.cursorignore`
- generated-file and sensitive-file exclusions
- semantic-document construction before vectorization
- local-only privacy status
- no cloud embedding or reranking calls

Cloud embeddings and cloud reranking remain out of scope for the default product.
Local metadata reranking is available only when provider policy enables it; it
uses candidate metadata and signal provenance, not raw source text.
Semantic status and plans now include a `providerPolicy` object that records
allowed, denied, unavailable, disabled, and skipped backend decisions. Local
source-free metadata providers are allowed by default; cloud embeddings, cloud
reranking, source transfer, and reranking execution are denied or disabled until
repo policy explicitly opts in.

Promotion to a default retrieval signal is controlled by `ctxhelm eval gate`.
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

The release gate also runs `scripts/smoke-semantic.sh` and checks that source and secret sentinels are not persisted in ctxhelm local state.
