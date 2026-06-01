# Phase 157: Benchmark Corpus Health Guard

## Goal

Prevent large-history R&D proofs from silently using a dirty, corrupt, or
wrong-revision local checkout. Produce source-free fixture-health evidence
before using RefactoringMiner or similar repositories as benchmark/product-proof
inputs.

## Implementation

- Added `scripts/prepare-benchmark-corpus.sh`.
- The script prepares a detached worktree for a requested corpus revision.
- Non-refresh runs fail on dirty existing fixtures.
- `--refresh` treats the fixture as disposable and fetches, hard-resets, cleans,
  and checks out the requested revision.
- The JSON health report records readiness, revision identity, commit count,
  dirty count, history usability, object-store connectivity, refresh status, and
  privacy metadata.
- The report omits source snippets, commit subjects, diffs, terminal logs, and
  prompts.

## RefactoringMiner Fixture Proof

Command:

```bash
bash scripts/prepare-benchmark-corpus.sh \
  --name RefactoringMiner \
  --source https://github.com/tsantalis/RefactoringMiner.git \
  --revision e319af8d6b51d821b61d2f735ad211631775adfb \
  --worktree /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --min-commits 20 \
  --output .ctxhelm/e2e/phase157-refactoringminer-corpus-health.json \
  --refresh
```

Result:

- Status: `ready`
- Head commit: `e319af8d6b51d821b61d2f735ad211631775adfb`
- Commit count: `5819`
- Dirty count: `0`
- Git history usable: `true`
- Object content usable: `true`
- Source text logged: `false`

## Backend Comparison Evidence

Command:

```bash
cargo run --release -p ctxhelm --locked -- eval lexical corpus \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 20 \
  --budget 10 \
  --format json > .ctxhelm/e2e/phase157-refactoringminer-lexical-backend-corpus.json
```

Result on the 20-commit sample:

| Metric | BM25 | Legacy | Delta |
| --- | ---: | ---: | ---: |
| Recall@5 | 0.075 | 0.100 | -0.025 |
| Recall@10 | 0.1125 | 0.175 | -0.0625 |
| MRR@10 | 0.11333333 | 0.13916667 | -0.025833338 |
| Total backend millis | 63691 | 931 | -62760 |

Comparison summary:

- BM25 wins@10: `1`
- Legacy wins@10: `3`
- Ties@10: `16`
- Average overlap@K: `1.95`
- Top path changed rate: `0.9`

## Interpretation

This phase improves proof reliability rather than retrieval quality. It also
surfaces an important R&D bottleneck: the current query-time BM25 path trails the
legacy scanner on this clean RefactoringMiner sample and is materially slower.
The next retrieval-quality phase should focus on backend scoring/runtime rather
than claiming BM25 lift.

## Validation

- `bash -n scripts/prepare-benchmark-corpus.sh`
- `cargo test -p ctxhelm --test release_packaging prepare_benchmark_corpus_script_contract_and_dirty_guard --locked`
