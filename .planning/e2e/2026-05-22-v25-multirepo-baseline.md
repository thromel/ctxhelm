# v2.5 Multi-Repo Baseline E2E

Date: 2026-05-22

Purpose: prove Phase 61 can evaluate retrieval quality across more than one real repository using the existing source-free benchmark-suite path.

## Command

```bash
ctxhelm eval benchmark --config .ctxhelm/e2e/v25-multirepo-baseline-config.json --format json
```

The full JSON report is intentionally kept under ignored `.ctxhelm/e2e/`.

## Suite

- Manifest version: `ctxhelm-benchmark-corpus-v2.5`
- Suite: `v2.5-multi-repo-baseline-smoke`
- Corpus ID: `v2.5-multi-repo-baseline-2026-05-22`
- Repositories configured: 2
- Repositories evaluated: 2
- Commits evaluated: 20
- Privacy: local-only
- Remote embeddings: false
- Remote reranking: false

## Results

| Repo | Commits | Default Recall@10 | Lexical Recall@10 | Lift@10 | Source Recall@10 | Test Recall@10 | Runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RefactoringMiner | 10 | 0.7767 | 0.7792 | -0.0025 | 0.8519 | 0.0000 | 18.4s |
| ctxhelm | 10 | 0.2270 | 0.2742 | -0.0472 | 0.2618 | 0.0000 | 7.7s |

## Gap Families

RefactoringMiner:

- `test` / `lexical_only_miss` / `src/test/java/org/refactoringminer/mcp/*.java` / testMapping
- `config` / `no_candidate_signal` / `docker/*` / storage
- `docs` / `no_candidate_signal` / `docker/*.md` / storage
- `source` / `lexical_only_miss` / `src/main/java/gr/uom/java/xmi/*.java` / lexicalRanking

ctxhelm:

- `docs` / `no_candidate_signal` / `docs/*.md` / storage
- `source` / `ranked_below_budget_dependency` / `crates/ctxhelm-compiler/src/*.rs` / parserPrecision
- `unknown` / `no_candidate_signal` / `scripts/*.sh` / storage
- `docs` / `no_candidate_signal` / `.planning/*.md` / storage

## Interpretation

The multi-repo benchmark path works and remains source-free. The evidence is also clear: ctxhelm should not claim quality lift yet. RefactoringMiner is near lexical parity, while ctxhelm trails lexical more clearly. Phase 62 should evaluate production local embeddings on the same corpus; Phase 64 should target the repeated docs/scripts/planning and compiler-source gap families.
