# Policy And Embedding Controls

ctxpack keeps semantic retrieval and cloud model use explicit. Default context
planning remains local lexical, graph, tests, history, memory, and feedback
signals. Local semantic retrieval is opt-in, and cloud embeddings/reranking stay
disabled unless a future policy gate explicitly enables them.

Phase 59 adds an explicit provider policy report to semantic status, context
plans, packs, and retrieval-policy experiment reports. The default policy is:

```json
{
  "allowLocalProviders": true,
  "allowCloudEmbeddings": false,
  "allowCloudReranking": false,
  "allowSourceTransfer": false,
  "enableLocalFixtureReranker": false
}
```

This means local source-free metadata providers such as `local_hash` can run
when explicitly requested, while cloud embedding providers, cloud rerankers, and
source-transfer paths are denied by default. Reranking is represented as a
typed provider decision but remains disabled unless `.ctxpack/provider-policy.json`
enables the deterministic local fixture reranker for testing.

Inspect the local semantic provider:

```bash
ctxpack semantic status --repo /path/to/repo --query "payment webhook validation"
```

The default provider is `local_hash` with model ID `ctxpack-local-hash-v1`.
It is deterministic scaffold/test behavior, source-free in reports, and exists
to make semantic policy paths testable without enabling cloud embeddings or
cloud reranking.

The optional production-local backend is `local_fastembed`. It is available only
in builds compiled with the `local-embeddings` Cargo feature. When that feature
is absent, requesting `local_fastembed` reports `semantic_provider_unavailable`
and keeps `remoteEmbeddingsUsed: false`.

## Provider Policy Shape

Optional repo policy lives at:

```text
.ctxpack/provider-policy.json
```

Example local fixture policy:

```json
{
  "schemaVersion": 1,
  "name": "local-reranker-fixture",
  "allowLocalProviders": true,
  "allowCloudEmbeddings": false,
  "allowCloudReranking": false,
  "allowSourceTransfer": false,
  "enableLocalFixtureReranker": true,
  "sourceTextLogged": false
}
```

Cloud settings still require the team privacy policy to allow cloud use and
source transfer; absent or restrictive team policy keeps remote provider
decisions denied. Policy-denied paths emit structured decisions and diagnostics
instead of silently pretending a backend ran.

Compare retrieval-policy experiment rows without changing default ranking:

```bash
ctxpack eval policy experiments "fix requireSession bug" --repo /path/to/repo
```

The policy experiment report includes lexical-only, hybrid local semantic,
graph neighborhood, current default rows, and the provider policy report. It is
report-only and includes `sourceTextLogged: false`.

## Source Boundary

These reports are metadata-only. They include provider kind, model ID,
dimensions, provider role, quality-backend status, local-only status, provider
availability, cache/freshness/degraded status, vector counts, semantic usage
counts, graph node/edge counts, recall metrics, diagnostics, and explicit
cloud-disabled flags, provider decisions, reranker decisions, and data-class
permissions. They do not include raw source text, prompts, terminal logs, model
transcripts, cloud payloads, or vector-provider request bodies.

## Smoke Test

Maintainers can run:

```bash
bash scripts/smoke-policy-embedding.sh
```

The smoke verifies local semantic provider status, provider-policy decisions,
cloud-disabled policy flags, policy experiment rows, disabled default reranking,
and absence of source sentinel leakage.
