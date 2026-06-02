# Phase 162: Feature-Enabled Local Fastembed Gate Proof

## Goal

Run the semantic/precision gate against the real production-local embedding
backend, not just the deterministic `local_hash` scaffold or a feature-disabled
provider request. Use the contribution diagnostics from Phase 161 to decide
whether `local_fastembed` is adding target evidence beyond lexical search.

## Implementation

- Added semantic-contribution diagnostics to the gate:
  - `semantic_contribution_no_candidates`
  - `semantic_contribution_no_unique_target_hits`
  - `semantic_contribution_unique_target_hits`
- Documented the diagnostic codes in `docs/benchmarking.md`.
- Kept the diagnostics source-free: paths are only included for named
  semantic-only target hits already present in the eval report.

## RefactoringMiner Proof

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Compile proof:

```bash
cargo check -p ctxhelm --features local-embeddings --locked
```

Result: passed.

Cold feature-enabled gate run:

```bash
CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 cargo run -p ctxhelm --features local-embeddings --locked -- \
  eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 1 \
  --budget 10 \
  --semantic-provider local_fastembed \
  --format json > /tmp/ctxhelm-phase162-refminer-gate-fastembed-feature.json
```

Result:

```text
total wall time: 5:24.46
decision: hold
reason: Held: runtime ratio 142.37x with recall delta +0.000; promotion requires stronger quality lift.
semanticOnlyTargetHitCount: 0
semanticLexicalOverlapCount: 2
```

Warm feature-enabled gate run after compile/model cache:

```bash
CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 target/debug/ctxhelm \
  eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 1 \
  --budget 10 \
  --semantic-provider local_fastembed \
  --format json > /tmp/ctxhelm-phase162-refminer-gate-fastembed-feature-diagnostic.json
```

Observed source-free summary:

```json
{
  "decision": "hold",
  "decisionReason": "Held: runtime ratio 3.34x with recall delta +0.000; promotion requires stronger quality lift.",
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
    "providerStatus": "local_fastembed",
    "fileRecallAt10": 1.0,
    "runtimeMillis": 2960
  },
  "diagnosticCodes": [
    "provider_policy_absent_safe_defaults",
    "provider_policy_remote_denied",
    "semantic_contribution_no_unique_target_hits"
  ]
}
```

Diagnostic:

```json
{
  "code": "semantic_contribution_no_unique_target_hits",
  "severity": "info",
  "message": "Semantic provider `local_fastembed` hit target files, but none were semantic-only target hits beyond the lexical baseline top K.",
  "paths": [],
  "count": 1
}
```

## Interpretation

The production-local backend works end-to-end in a feature-enabled build and
stays local/source-free. On this clean RefactoringMiner gate sample, it does not
justify promotion: it preserves target coverage but adds no semantic-only target
evidence beyond lexical and remains slower than default even after warmup.

The next R&D work should target either:

- better task/query construction for conceptual semantic retrieval, or
- lower-overhead local embedding reuse/cache behavior before larger-corpus
  `local_fastembed` gates.

## Validation

- `cargo check -p ctxhelm --features local-embeddings --locked`
- `cargo test -p ctxhelm-compiler semantic_contribution --locked`
- cold feature-enabled `local_fastembed` gate on clean RefactoringMiner
- warm feature-enabled `local_fastembed` gate on clean RefactoringMiner
- `cargo fmt --check`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm --locked -- --help`
- `cargo test -p ctxhelm-compiler semantic --locked`
- `cargo test -p ctxhelm-index semantic --locked`
- `cargo test --workspace --locked --no-fail-fast`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `git diff --check`
