# Phase 160: Bounded Semantic Status And Search

## Goal

Make the direct semantic status/search surfaces usable on a large real
repository before investing further in semantic-quality experiments. The target
fixture is the clean upstream RefactoringMiner clone prepared for the Phase 157
and Phase 159 proof work.

This phase intentionally does not claim that the default `prepare-task` planner
is fast on large repos. The default planner still has a broader latency
bottleneck that must be handled separately.

## Root Cause

The semantic status/search code path was paying for more work than the requested
surface needed:

- `semantic_document_report` built full symbol, dependency, and related-test
  enrichment before truncating documents.
- Query status ran the full `prepare_task` semantic path to estimate usage.
- Status materialized local vectors just to count them.
- Direct semantic search scored only the local deterministic embedding scaffold,
  so exact path and identifier evidence could be ignored after candidate
  prefiltering.

On clean RefactoringMiner, this made semantic status take around a minute for a
provider-only check, query status stretch past two minutes in one probe, and the
direct semantic query `Improvement in TypeScriptVisitor` initially ranked the
target file third.

## Implementation

- Added `SemanticDocumentOptions` controls for query prefiltering and optional
  symbol, dependency, and related-test enrichment.
- Truncated semantic document candidates before per-file related-test
  enrichment.
- Added source-free `path` metadata facets to semantic documents.
- Made direct semantic search request a bounded source-free document sample
  instead of all semantic documents.
- Added an exact metadata boost to final semantic scoring so exact path and
  identifier evidence can break deterministic local-vector ties.
- Changed query semantic status to use a bounded `semantic_search` sample rather
  than a full `prepare_task` plan.
- Avoided eager local vector materialization in semantic status; persisted vector
  counts still come from storage status.
- Updated semantic precision gate document sampling to use the cheap source-free
  document path.

## RefactoringMiner Proof

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Commands:

```bash
target/debug/ctxhelm semantic status \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --query "Improvement in TypeScriptVisitor" \
  --format json > /tmp/ctxhelm-phase160-localhash-status-final.json

target/debug/ctxhelm semantic status \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic-provider local_fastembed \
  --format json > /tmp/ctxhelm-phase160-fastembed-status-final.json

target/debug/ctxhelm search \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic \
  --limit 10 \
  "Improvement in TypeScriptVisitor" > /tmp/ctxhelm-phase160-semantic-search-final.json
```

Observed results:

| Probe | Before | Phase 160 |
| --- | ---: | ---: |
| Provider-only semantic status | `1:09.34`, provider unavailable after full document enrichment | about `6.8s`, no eager vectors |
| Query semantic status | killed/slow probe, then `2:08.76` before removing full planner usage | about `6.9s`, bounded 10-result semantic sample |
| Direct semantic search | about `48s`, target ranked third | about `6.8s`, target ranked first |

Final query status summary:

```json
{
  "semanticDocumentCount": 10,
  "semanticFacetCount": 10,
  "localVectorCount": 0,
  "storedVectorCount": 0,
  "usage": [
    {
      "surface": "semantic_search",
      "semanticEnabled": true,
      "semanticCandidateCount": 10,
      "remoteEmbeddingsUsed": false
    }
  ],
  "providerAvailable": true,
  "providerRole": "deterministic_scaffold"
}
```

Final direct semantic search top result:

```text
1. src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java
   score: 0.73726636
   reason: local semantic similarity via local_hash ctxhelm-local-hash-v1 over source-free facets: path; exact metadata boost 0.440
```

Provider-only status for `local_fastembed` remains correctly unavailable in the
current build, but now returns without full enrichment or eager vector work.

## Interpretation

Phase 160 makes semantic search/status cheap enough to use as diagnostic and
candidate-generation surfaces on a large repository. It also fixes a concrete
quality failure where source-free exact path metadata was visible during
prefiltering but not strong enough in final ranking.

This does not prove production semantic lift over lexical search. The active
local default still uses the deterministic `local_hash` scaffold unless a
production local embedding provider is enabled. The next semantic-quality work
should compare bounded local embedding providers against lexical and graph
baselines through the existing proof harness.

## Validation

- `target/debug/ctxhelm semantic status --query ...` on clean RefactoringMiner
- `target/debug/ctxhelm semantic status --semantic-provider local_fastembed ...` on clean RefactoringMiner
- `target/debug/ctxhelm search --semantic ...` on clean RefactoringMiner
- `cargo fmt --check`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm --locked -- --help`
- `cargo test -p ctxhelm-index semantic --locked`
- `cargo test -p ctxhelm-compiler semantic --locked`
- `cargo test -p ctxhelm --test cli_compat semantic --locked`
- `cargo test --workspace --locked --no-fail-fast`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `git diff --check`
