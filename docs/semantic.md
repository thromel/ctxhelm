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
- Jina code model status: still available explicitly as `JinaEmbeddingsV2BaseCode` with 768 dimensions and provider-specific `query:` / `passage:` text, but Phase 293 keeps it diagnostic-only because candidate quality improved without Recall@10 lift and runtime remained high. Phase 294 also rejects promoting semantic as post-top-K next-read guidance for that setup because the only appended next-read path was a non-target. Phase 295 rejects `AllMiniLML12V2Q` and `AllMiniLML12V2` as simple model-swap fixes because both are noisier than Jina on the same targeted proof. Phase 296 shows the remaining Jina candidate miss has `dependency_co_change` support. Phase 297's eval-only supported-candidate tail-slot oracle recovers that target cleanly, and Phase 298 replaces the oracle dependency with a source-free `symbol_identifier` / `python_source` / `dependency_co_change` supported-shape predictor that matches the clean lift on the targeted slice. Phase 299 broadens the predictor proof across eight slices and finds zero regressions but only the original VeriSchema older lift, so keep it eval-only.
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

Semantic document and query text include source-free identifier aliases, so
camel-case and acronym-heavy code names such as `UMLOperationDiff` also expose
tokens like `uml operation diff` to local embedding backends. The persisted
vector hash includes the semantic document render version, so changed
source-free render text does not reuse older vectors for the same file hash.

Large generated fixture trees are pruned before inventory and freshness walks
when generated files are excluded by policy. In the RefactoringMiner proof, that
cut a bounded `local_fastembed` seed from Phase 166 `55.65s` to `5.18s` and a
second fresh-process semantic search from `12.08s` to `0.11s` while preserving
the known `TypeScriptVisitor.java` top result. This is a latency improvement,
not a semantic-quality promotion; semantic retrieval remains opt-in until fixed
corpus gates show target-file recall lift over lexical.

Phase 168 also made semantic contribution diagnostics stricter. Gates now report
when semantic contributes files outside lexical that are not retrieval targets.
On the clean RefactoringMiner 3-commit `local_fastembed` gate, the only
semantic-only file was a non-target test, while target recall stayed flat against
lexical/default. That result keeps semantic opt-in and points next R&D toward
graph/history/fusion for coupled source files rather than broader semantic
promotion.

Phase 175 adds semantic-missed target gap families to the same gate. The report
now distinguishes semantic misses with nonsemantic graph/history/symbol signal
from misses with no candidate signal at all. On the clean RefactoringMiner
2-commit `local_hash` gate, semantic still held with recall delta `+0.000`; the
remaining semantic miss was `semantic_miss_area_context_only` for
`UMLClassBaseDiff.java`, while semantic-only additions were non-targets.

Phase 176 follows that result without overfitting semantic ranking. The
remaining RefactoringMiner miss did not have lexical, symbol, dependency,
co-change, or semantic evidence strong enough to justify top-K promotion.
Instead, standard task plans now expose focused context-area resources when the
selected source files identify an area with unselected source-like candidates.
On a fresh RefactoringMiner 2-commit proof, Recall@10 stayed `0.75` versus the
lexical baseline `0.5833334`, but each commit now surfaced
`ctxhelm://repo/context-area/src%2Fmain` as an actionable progressive read
resource for the `area_context_only` gap.

Phase 177 narrows that resource for JVM repositories. Maven/Gradle Java and
Kotlin paths now use package-level context areas, so the same RefactoringMiner
gap surfaces
`ctxhelm://repo/context-area/src%2Fmain%2Fjava%2Fgr%2Fuom%2Fjava%2Fxmi`
instead of broad `src/main` while keeping Recall@10 flat.

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

Phase 252 clarifies the current `local_fastembed` gate state. The real local
backend is available and source-free, but default promotion remains on hold:
RefactoringMiner shows a small semantic lift, while ctxhelm is neutral. The gate
now keeps eval-only local metadata reranker regressions visible without letting
those unrelated regressions misclassify semantic as blocked.

Phase 256 rejects richer `local_fastembed` search documents as a default path.
Adding source-free symbol and dependency facets to semantic search/index
documents preserved privacy but removed the RefactoringMiner semantic lift,
kept ctxhelm neutral, and materially increased semantic runtime. Do not promote
semantic by stuffing more facets into search documents. Future semantic R&D
should target task-conditioned query construction, alternate local model/fusion
evaluation, or safe local metadata reranker promotion constraints.

Phase 294 adds `semanticNextReadContribution` to semantic gate reports. This
source-free diagnostic preserves the full default top K and measures bounded
semantic next-read paths after that protected budget. The targeted VeriSchema
Jina candidate-path proof appended only one path,
`tests/core/test_state_validator.py`, and it was not a retrieval target, so the
gate emitted `semantic_next_read_noise_hold`. Treat the field as R&D evidence,
not runtime/default guidance.

Phase 296 adds candidate-missed support profiles to `semanticContribution`.
For semantic-generated retrieval targets that final top-K selection drops, the
report now groups their non-semantic source-free support families. On the
targeted VeriSchema Jina candidate-path proof, the only such missed target is
`schema_agent/core/state.py` with `dependency_co_change` support, yielding
`semantic_candidate_fusion_supported_gap`. Treat that as fusion/top-K ordering
evidence, not as a reason to expand semantic documents or promote Jina.

Phase 297 adds `semantic_supported_candidate_tail_slot_oracle` and
`supportedCandidateTailSlotRerankerContribution` to measure the upper bound of
that supported candidate-miss surface. On the targeted VeriSchema Jina proof,
the oracle recovers `schema_agent/core/state.py`, improves target hits
`13 -> 14`, and has no default-only target churn. It is explicitly eval-only
because it consumes `candidateMissedFileProfilesAt10`, which are built from eval
target misses.

Phase 298 adds the eval-only source-free
`semantic_supported_shape_tail_slot_reranked` variant and
`supportedShapeTailSlotSemanticRerankerContribution`. It uses generated
`supportedSemanticCandidateProfilesAt10` rows, not target-miss profiles, and
only inserts the measured `symbol_identifier` / `python_source` /
`dependency_co_change` supported shape into protected tail slots. On the same
targeted proof it matches the oracle's clean lift: `schema_agent/core/state.py`,
target hits `13 -> 14`, and no default-only target churn. Keep it eval-only
until broader range/repo proof shows the shape repeats.

Phase 299 runs that broader proof across RefactoringMiner, ctxhelm, ReAgent,
and VeriSchema older/recent ranges. Across 159 evaluated commits, the
supported-shape variant has one improved commit, zero regressions, and zero
default-only target hits. The only lift remains the original VeriSchema older
case, so this is safety/sparsity evidence, not runtime/default promotion
evidence.

## When To Avoid It

Avoid semantic retrieval for exact identifier, stack trace, route, config-key, or single-file edit tasks where lexical or explicit path anchors are stronger. Use it when the task is conceptual, such as finding payment webhook validation, retry logic, normalization flows, or analogous feature patterns.

## Validation

Maintainers can run deterministic local coverage with:

```bash
bash scripts/smoke-semantic.sh
```

The release gate also runs `scripts/smoke-semantic.sh` and checks that source and secret sentinels are not persisted in ctxhelm local state.
