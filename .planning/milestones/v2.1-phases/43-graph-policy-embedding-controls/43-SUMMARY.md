# Phase 43 Summary: Graph-Aware Policy & Embedding Controls

**Completed:** 2026-05-18
**Status:** Complete

## Delivered

- Added source-free semantic provider status contracts and CLI output through `ctxhelm semantic status`.
- Added report-only retrieval policy experiment rows through `ctxhelm eval policy experiments`.
- Compared lexical-only, hybrid local semantic, graph-neighborhood, and current disabled-semantic default policies without changing default ranking.
- Preserved explicit local-only privacy flags: cloud embeddings disabled, cloud reranking disabled, and `sourceTextLogged: false`.
- Added policy/embedding documentation and release-gate smoke coverage.

## Validation

- `cargo fmt --all --check`
- `bash scripts/smoke-policy-embedding.sh`
- `bash scripts/check-release-docs.sh`
- `bash -n scripts/release-gate.sh`
- `bash -n scripts/smoke-policy-embedding.sh`
- `cargo test -p ctxhelm release_gate_script_contract -- --nocapture`
- `cargo test -p ctxhelm release_docs_check_passes -- --nocapture`

## Notes

- This phase intentionally does not enable cloud embeddings, cloud reranking, or semantic ranking by default.
- Policy experiment rows are diagnostic and source-free; they do not mutate repo policy state.
