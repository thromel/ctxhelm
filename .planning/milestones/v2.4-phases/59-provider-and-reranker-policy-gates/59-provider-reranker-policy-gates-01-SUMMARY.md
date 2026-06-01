# Phase 59 Summary: Provider And Reranker Policy Gates

## Outcome

Phase 59 is complete.

ctxhelm now has explicit provider and reranker policy gates. Local source-free
semantic metadata providers remain allowed, while cloud embeddings, cloud
reranking, source transfer, and reranker execution remain denied or disabled by
default.

## Implemented

- Added typed provider policy contracts:
  - `ProviderPolicy`
  - `ProviderPolicyReport`
  - `ProviderDecision`
  - `ProviderCapability`
  - `ProviderDataClass`
  - `ProviderDecisionStatus`
- Attached provider policy reports to:
  - `ContextPlan`
  - `ContextPack`
  - `SemanticProviderStatusReport`
  - `RetrievalPolicyExperimentReport`
- Added `.ctxhelm/provider-policy.json` loading with safe defaults when absent.
- Kept existing team privacy policy as an additional gate for cloud/source
  transfer.
- Enforced semantic provider decisions before semantic retrieval runs.
- Added a deterministic local fixture reranker behind policy gates.
- Kept reranking disabled by default.
- Added provider-policy sections to generated context packs and CLI markdown
  reports.
- Updated policy/embedding smoke coverage to assert provider decisions,
  cloud-denied flags, source-transfer denial, and disabled default reranking.
- Updated docs for provider policy shape, reranker defaults, and release proof
  expectations.

## Verification

- `cargo test -p ctxhelm-core policy --no-fail-fast`
- `cargo test -p ctxhelm-compiler policy --no-fail-fast`
- `cargo test -p ctxhelm-compiler rerank --no-fail-fast`
- `cargo test -p ctxhelm --test cli_compat policy --no-fail-fast`
- `cargo test --workspace --no-fail-fast`
- `CTXHELM_BIN=target/debug/ctxhelm bash scripts/smoke-policy-embedding.sh`
- `CTXHELM_BIN=target/debug/ctxhelm bash scripts/smoke-semantic.sh`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`

## Notes

- The policy report is source-free. Structured JSON uses the typed
  `sourceTextAllowed` field, while markdown renderers avoid source-bearing
  terminology guarded by existing source-free pack tests.
- The local fixture reranker is intended for deterministic policy/eval tests,
  not as a default quality backend.
- Cloud provider paths remain blocked unless both provider policy and team
  privacy policy allow them.
