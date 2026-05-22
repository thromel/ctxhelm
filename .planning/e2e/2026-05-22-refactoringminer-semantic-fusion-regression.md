# RefactoringMiner Semantic Fusion Regression Check

Date: 2026-05-22

Repo under test: `../RefactoringMiner`

Command shape:

```bash
cargo run -p ctxpack -- eval baselines --repo ../RefactoringMiner --limit 20 --budget 10 --parallelism 4 --force --format json
cargo run -p ctxpack -- eval baselines --repo ../RefactoringMiner --limit 20 --budget 10 --parallelism 4 --force --semantic --semantic-provider local_hash --format json
```

## Issue Found

The first rerun exposed two separate problems:

1. RefactoringMiner contained Finder-style duplicate source folders such as `src/main 2/` and `src/test 2/`. These polluted inventory scanning and made the historical eval look hung.
2. `local_hash` semantic candidates were being used as graph/test expansion seeds. On commit `c16c031d048618c8389f474f80c2feee21a06ad1`, semantic seeded `Constants.java`, graph expansion displaced the gold changed file `MethodMatcher.java`, and semantic-enabled Recall@10 regressed.

## Fixes

- Added generated-file policy exclusions for duplicate source copy folders:
  - `src/main 2/`
  - `src/test 2/`
- Made source-free target selection keep a stronger lexical floor before exploratory graph/semantic candidates.
- Aligned eval context ranking with pack ordering: target files first, validation tests when the file budget has room.
- Restricted graph/test expansion seeds to anchor, symbol, and lexical paths. Semantic paths are used as expansion seeds only when no exact seeds exist.

## Results

| Run | Recall@10 | Precision@10 | MRR@10 | Delta vs lexical | Runtime |
| --- | ---: | ---: | ---: | ---: | ---: |
| Before default | 0.6222 | 0.1950 | 0.7100 | -0.0443 | 44.2s |
| After lexical floor default | 0.6355 | 0.1850 | 0.7033 | -0.0310 | 31.4s |
| After semantic seed fix default | 0.6355 | 0.1850 | 0.7033 | -0.0310 | 32.2s |
| Before `local_hash` | 0.6172 | 0.1950 | 0.7100 | -0.0493 | 62.3s |
| After lexical floor `local_hash` | 0.6105 | 0.1800 | 0.7033 | -0.0560 | 65.7s |
| After semantic seed fix `local_hash` | 0.6355 | 0.1850 | 0.7033 | -0.0310 | 68.8s |

Semantic-only signal also improved:

| Run | Semantic-only Recall@10 |
| --- | ---: |
| Before `local_hash` | 0.2826 |
| After lexical floor `local_hash` | 0.2893 |
| After semantic seed fix `local_hash` | 0.3143 |

## Current Interpretation

The semantic integration now works without hurting the combined ranking, but it does not yet improve the combined ranking beyond the lexical-heavy default. The remaining gap is quality: `local_hash` is a deterministic local scaffold, not a strong embedding backend. The next product milestone should focus on production local embeddings and better learned fusion/reranking rather than more plumbing.

Evidence JSON:

- `.ctxpack/e2e/refminer-baselines-default-semseedfix-2026-05-22.json`
- `.ctxpack/e2e/refminer-baselines-local-hash-semseedfix-2026-05-22.json`
