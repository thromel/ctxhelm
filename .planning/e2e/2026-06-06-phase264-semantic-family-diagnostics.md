# Phase 264: Semantic Query-Family Diagnostics

## Scope

Phase 264 advances the remaining semantic/fusion R&D risk. The goal is to stop
judging semantic retrieval only as a repo-wide aggregate and instead expose
which source-free query families show unique target lift, noise, or missed
targets.

Claude Code paired outcome proof was retried first through the source-free
availability preflight. Claude Code `2.1.163` is still currently rate-limited,
so no new Claude retrieval-quality conclusion is drawn from this phase. Codex
CLI remains available.

## Implementation

- Added `semanticContribution.queryFamilyContributions`.
- Added per-family semantic counts:
  - evaluated commits
  - commits with semantic selection
  - semantic-selected files
  - semantic target hits
  - semantic-only target hits beyond lexical
  - semantic-only non-targets beyond lexical
  - semantic missed targets
  - missed-target gap families
  - source-free example cases
- Added diagnostics:
  - `semantic_query_family_route_candidate`
  - `semantic_query_family_mixed_hold`
  - `semantic_query_family_noise_hold`
- Rendered semantic query-family contributions in markdown gate reports.

## Evidence

Artifacts:

- `.ctxhelm/e2e/phase264-agent-client-availability.json`
- `.ctxhelm/e2e/phase264-semantic-family-refactoringminer.json`
- `.ctxhelm/e2e/phase264-semantic-family-ctxhelm.json`
- `.ctxhelm/e2e/phase264-semantic-family-reagent.json`
- `.ctxhelm/e2e/phase264-semantic-family-verischema.json`

Commands:

```bash
CTXHELM_BIN=/Users/romel/Documents/GitHub/ctxhelm/target/debug/ctxhelm \
  bash scripts/e2e-agent-client-availability.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm \
  --task "Improve semantic routed reranker policy" \
  --output .ctxhelm/e2e/phase264-agent-client-availability.json

cargo build -p ctxhelm --features local-embeddings

./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase264-semantic-family-refactoringminer.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase264-semantic-family-ctxhelm.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase264-semantic-family-reagent.json
./target/debug/ctxhelm eval gate --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema --limit 10 --budget 10 --semantic-provider local_fastembed --format json > .ctxhelm/e2e/phase264-semantic-family-verischema.json
```

## Findings

| Repo | Decision | Default Recall@10 | Semantic Recall@10 | Family finding |
| --- | --- | ---: | ---: | --- |
| RefactoringMiner | hold | 0.5383333 | 0.5583333 | `symbol_identifier` has 1 semantic-only target but 9 semantic-only non-targets, so it is a mixed hold. |
| ctxhelm | hold | 0.62736285 | 0.62736285 | `broad_scope` has 2 semantic-only targets and 0 semantic-only non-targets, but 63 missed targets remain. |
| ReAgent | hold | 0.5 | 0.5 | `commit_clue`, `symbol_identifier`, and `domain_phrase` all show semantic-only non-targets without unique target hits. |
| VeriSchema | block | 0.48764706 | 0.48764706 | `domain_phrase` has 3 semantic-only targets and 0 semantic-only non-targets; `broad_scope` is mixed with 3 targets and 1 non-target. |

## Result

Semantic retrieval is still not default-promotable. The useful R&D direction is
selective semantic/fusion routing:

- Candidate families: `domain_phrase` on VeriSchema, `broad_scope` on ctxhelm.
- Hold/noise families: ReAgent `commit_clue`, ReAgent `symbol_identifier`,
  ReAgent `domain_phrase`, and RefactoringMiner `symbol_identifier`.

The next semantic R&D should validate whether route candidates repeat on larger
samples before adding runtime semantic routing.
