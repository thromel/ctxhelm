# Phase 263: Routed Reranker Runtime Policy

## Scope

Phase 262 proved that the safe routed policy is `commit_clue` only. Phase 263
turns that evidence into opt-in runtime behavior without changing default
ranking.

## Implementation

- Added `ProviderPolicy.enableQueryFamilyRoutedReranker`.
- Added a `local_metadata_routed` provider-policy decision for the routed path.
- Shared the route predicate between eval and runtime policy:
  `commit_clue` is route-enabled; `symbol_identifier`, `domain_phrase`,
  `explicit_path`, and broad/low-information families remain held.
- Planner runtime now applies local metadata reranking only when:
  - the repo policy enables `enableQueryFamilyRoutedReranker`,
  - local providers are allowed, and
  - the task query family is `commit_clue`.
- Unproven families keep default ranking and emit
  `query_family_routed_reranker_held`.

## Evidence

Artifact:

- `.ctxhelm/e2e/phase263-routed-reranker-runtime-policy.json`

CLI smoke:

```bash
cargo run -q -p ctxhelm -- prepare-task --repo "$repo" --mode bug-fix "update payment retry behavior"
cargo run -q -p ctxhelm -- prepare-task --repo "$repo" --mode bug-fix "fix AuthService redirect failure"
```

Focused tests:

```bash
cargo test -p ctxhelm-compiler policy::tests::reranker_decision_is_disabled_by_default_and_local_when_enabled --locked
cargo test -p ctxhelm-compiler query_family_routed_reranker --locked
```

## Result

The `commit_clue` runtime branch reports
`query_family_routed_reranker_applied`, with provider `local_metadata_routed`,
metadata-only data class, no remote provider, and no source text transfer.

The `symbol_identifier` branch reports
`query_family_routed_reranker_held` while keeping the same local-only,
source-free policy posture.

This closes the Phase 262 policy decision: routed reranking is now available as
an explicit local opt-in, but default behavior and full metadata reranking
remain unchanged.
