# Phase 213: Report-Level R&D Action Routing

Date: 2026-06-03

## Goal

Extend Phase 212 `recommendedResearchActions` beyond paired real-agent reports
into historical eval and product-proof reports, keeping the field source-free
and machine-checkable.

## What Changed

- Added `RecommendedResearchAction` to compiler report contracts with:
  - `action`
  - `priority`
  - `origin`
  - `reason`
- Added `recommendedResearchActions` to `HistoricalEvalReport`.
- Added `recommendedResearchActions` to `ProductProofReport`.
- Historical eval reports now route:
  - no-candidate misses to candidate-generation work
  - generated-but-unselected misses to ranking or budget allocation
  - context-area next-read recovery to progressive native-read guidance
  - agent-evidence-only recovery to next-read alignment
  - validation/test gaps to validation-test mapping
  - protected-evidence misses to protected-evidence preservation
  - graph edge ablations/profiles to graph edge-budget work
- Product-proof reports now route:
  - insufficient corpus evidence to fixture/history refresh
  - runtime notes to runtime/cache work
  - protected target misses to protected-evidence preservation
  - trailing/unexplained lexical claims to retrieval/ranking fixes
  - explained all-file lexical trails to native-baseline analysis
  - missing BM25 evidence to backend evidence collection
  - clean promoted reports to preserve-contract actions
- Markdown renderers now show a `Recommended R&D Actions` section for
  historical eval and product-proof reports.

## Source-Free Guard Fix

The first full workspace test run failed because the new action field was named
`source`, which correctly tripped the existing source/prompt text field guard.
The field was renamed to `origin` instead of weakening the guard.

## Validation

- `cargo test -p ctxhelm-compiler --lib historical_eval_report_public_json_shape_is_stable --locked`
- `cargo test -p ctxhelm --bin ctxhelm historical_eval_report_renders_source_free_metrics --locked`
- `cargo test -p ctxhelm-compiler --lib product_proof_release_gate --locked`
- `cargo fmt --check`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm --locked -- --help`
- `cargo run -p ctxhelm --locked -- eval history --limit 1 --format json | rg 'recommendedResearchActions|origin|improve_candidate_generation|improve_ranking_or_budget_allocation'`
- `cargo test -p ctxhelm --test cli_compat --locked`
- `cargo test --workspace --locked --no-fail-fast`
- `CTXHELM_ALLOW_DIRTY=1 bash scripts/release-gate.sh`

All commands passed after the `source` to `origin` field rename.

## Current Proof

The live historical eval JSON includes `recommendedResearchActions` with
`origin = "historical_eval"` and actions including:

- `improve_candidate_generation`
- `improve_ranking_or_budget_allocation`

The release gate passed, including workspace tests, release docs consistency,
release package/audit, archive verification, smoke tests, deterministic MCP
protocol checks, release governance, semantic/precision gates, and release
proof-bundle generation.
