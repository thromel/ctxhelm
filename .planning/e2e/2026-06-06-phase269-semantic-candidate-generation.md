# Phase 269 - Semantic Candidate Generation Diagnostics

## Scope

Separate semantic candidate-generation failures from semantic fusion failures.
Phase 268 showed support-profile routing was safe but neutral, so the next
question is whether semantic retrieval fails because it never generates the
right target candidates or because generated semantic candidates lose final
top-K selection.

This phase adds source-free diagnostics only. It does not change runtime
provider policy, default ranking, semantic documents, or reranker behavior.

## Implementation

`semanticContribution` now reports:

- `semanticCandidateTargetHitCount`
- `semanticCandidateMissedTargetCount`
- `semanticCandidateTargetHitRate`
- `semanticCandidateMissedTargetPaths`

The same candidate counters are included per query family. Gate diagnostics now
emit:

- `semantic_candidate_generation_gap` when semantic missed target files before
  candidate selection;
- `semantic_candidate_fusion_gap` when semantic generated source-free target
  candidates that final top-K selection dropped.

## Validation Commands

```bash
cargo fmt --check
cargo test -p ctxhelm-compiler semantic_contribution --locked
cargo test -p ctxhelm-compiler semantic_candidate --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run -q -p ctxhelm -- --help
bash scripts/check-release-docs.sh
git diff --check
cargo build -p ctxhelm --features local-embeddings
```

Four-repo gate refresh:

```bash
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase269-semantic-candidate-generation-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase269-semantic-candidate-generation-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase269-semantic-candidate-generation-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 20 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase269-semantic-candidate-generation-verischema.json
```

## Results

| Repo | Decision | Default Recall@10 | Local Semantic Recall@10 | Selected semantic targets | Candidate semantic targets | Candidate missed targets | Diagnostic |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| ctxhelm | `hold` | `0.44620585` | `0.44620585` | `7` | `24` | `17` | `semantic_candidate_fusion_gap` |
| ReAgent | `hold` | `0.35` | `0.35` | `2` | `5` | `3` | `semantic_candidate_fusion_gap` |
| RefactoringMiner | `hold` | `0.41857147` | `0.4285714` | `9` | `12` | `3` | `semantic_candidate_fusion_gap` |
| VeriSchema | `block` | `0.39382353` | `0.39382353` | `11` | `11` | `0` | `semantic_candidate_generation_gap` |

## Decision

Do not run another broad semantic routing pass yet.

Phase 269 shows that the next semantic work is not one-size-fits-all:

- ctxhelm, ReAgent, and RefactoringMiner need fusion, ranking, or budget
  experiments because semantic already generated additional target candidates
  that did not survive final top-K selection.
- VeriSchema needs query construction, model choice, or document coverage work
  because semantic did not generate additional missed target candidates.

Semantic remains opt-in. Any promotion candidate must beat the Phase 268 neutral
support-profile route on recall, churn, named regressions, and source-free
diagnostics.
