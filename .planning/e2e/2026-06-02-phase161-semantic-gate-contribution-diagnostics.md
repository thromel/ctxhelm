# Phase 161: Semantic Gate Contribution Diagnostics

## Goal

Make the semantic/precision gate answer the R&D question that matters before
promotion: is semantic retrieval contributing target evidence beyond lexical
search, or is it only overlapping the existing lexical/default candidate set?

Phase 160 made direct semantic status/search bounded and fixed one exact-path
semantic ranking miss. Phase 161 turns that into better evaluation evidence.

## Implementation

- Added `SemanticContributionSummary` to `SemanticPrecisionGateReport`.
- The summary reports:
  - evaluated commits
  - commits with semantic-selected files
  - semantic-selected file count
  - semantic target hits
  - semantic-only target hits absent from the lexical baseline top K
  - semantic/lexical overlap
  - semantic missed target count
  - average semantic-selected files
  - semantic target hit rate
  - semantic-only target hit rate
  - source-free named semantic-only target hit cases
- Rendered the contribution summary in `ctxhelm eval gate` Markdown output.
- Added `--semantic-provider`, `--semantic-model`, and
  `--semantic-dimensions` support to `ctxhelm eval gate`.
- Added `semantic_precision_gate_report_with_provider` while preserving the
  existing default-provider API.
- Documented provider-aware semantic gate usage in `docs/benchmarking.md`.

## RefactoringMiner Proof

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Default scaffold provider:

```bash
target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 1 \
  --budget 10 \
  --format json > /tmp/ctxhelm-phase161-refminer-gate-localhash.json
```

Observed source-free summary:

```json
{
  "decision": "hold",
  "evaluatedCommits": 1,
  "semanticContribution": {
    "commitsWithSemanticSelection": 1,
    "semanticSelectedFileCount": 2,
    "semanticTargetHitCount": 1,
    "semanticOnlyTargetHitCount": 0,
    "semanticLexicalOverlapCount": 2,
    "semanticMissedTargetCount": 0,
    "averageSemanticSelectedFiles": 2.0,
    "semanticTargetHitRate": 1.0,
    "semanticOnlyTargetHitRate": 0.0
  },
  "localSemantic": {
    "providerStatus": "local_hash",
    "fileRecallAt10": 1.0,
    "runtimeMillis": 916
  }
}
```

Interpretation: semantic helped preserve target coverage on the sampled commit,
but every semantic-selected file overlapped lexical and there were no
semantic-only target hits. The gate correctly stayed `hold`.

Feature-disabled production provider request:

```bash
target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 1 \
  --budget 10 \
  --semantic-provider local_fastembed \
  --format json > /tmp/ctxhelm-phase161-refminer-gate-fastembed-disabled.json
```

Observed source-free summary:

```json
{
  "decision": "hold",
  "semanticContribution": {
    "commitsWithSemanticSelection": 0,
    "semanticSelectedFileCount": 0,
    "semanticTargetHitCount": 0,
    "semanticOnlyTargetHitCount": 0,
    "semanticLexicalOverlapCount": 0,
    "semanticMissedTargetCount": 1
  },
  "localSemantic": {
    "providerStatus": "local_fastembed",
    "fileRecallAt10": 1.0,
    "runtimeMillis": 877
  }
}
```

This proves the gate now routes provider selection, while the default build does
not silently claim production-local embedding contribution.

Feature compile proof:

```bash
cargo check -p ctxhelm-index --features local-embeddings --locked
```

Result: passed in `1m 19s`.

## Interpretation

Phase 161 does not claim semantic beats lexical. It makes that question more
measurable. A future production-local embedding run can now use the same gate
with `--features local-embeddings` and `--semantic-provider local_fastembed`,
then inspect semantic-only target hits rather than relying on aggregate recall
alone.

## Validation

- `cargo run -p ctxhelm --locked -- eval gate --help`
- `cargo test -p ctxhelm-compiler semantic_contribution_summary_counts_semantic_only_target_hits --locked`
- `cargo test -p ctxhelm-compiler semantic --locked`
- `target/debug/ctxhelm eval gate --format markdown` on clean RefactoringMiner
- `target/debug/ctxhelm eval gate --format json` on clean RefactoringMiner
- `target/debug/ctxhelm eval gate --semantic-provider local_fastembed --format json` on clean RefactoringMiner
- `cargo check -p ctxhelm-index --features local-embeddings --locked`
- `cargo fmt --check`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm --locked -- --help`
- `cargo test -p ctxhelm-compiler semantic --locked`
- `cargo test -p ctxhelm-index semantic --locked`
- `cargo test -p ctxhelm --test cli_compat eval --locked`
- `cargo test --workspace --locked --no-fail-fast`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `git diff --check`
