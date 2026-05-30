# Phase 99: Context Area Read Batches

## Goal

Make broad context-area resources useful as progressive agent inputs without
adding MCP tools, exposing source text, or perturbing target-file selection.

Phase 98 left the next production-readiness path clear: top-10 target-file
churn is risky, but broad tasks still need better source-free guidance for the
areas that are only surfaced through `contextAreas`. Dynamic area resources now
need enough structure for agents to decide what to read next natively.

## Implementation

- Added source-free `roleBuckets` to dynamic
  `ctxpack://repo/context-area/{encoded-area}` resources.
- Added `nextReadBatches` with primary source/config/schema paths, validation
  test paths, and docs paths.
- Added `pathFamilies` to static and dynamic context-area resources so agents
  can see area shape without source text.
- Kept MCP tools unchanged.
- Kept resource payloads local-only and source-free; paths and counts are
  returned, not snippets.

## Evidence

Focused tests:

```bash
cargo test -p ctxpack-mcp resources_public_uri_shapes_are_stable -- --nocapture
cargo test -p ctxpack-mcp resource_read_returns_repo_summary_and_test_map -- --nocapture
```

Broad proof:

```bash
cargo run --release -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > .ctxpack/e2e/phase99-context-area-read-batches-proof.json

python3 scripts/check-product-proof.py \
  .ctxpack/e2e/phase99-context-area-read-batches-proof.json
```

Committed proof:

- `.ctxpack/e2e/phase99-context-area-read-batches-proof.json`

Result:

- `releaseGate.decision = promote`
- No metric deltas versus Phase 98 for File Recall@10, Source Recall@10, Test
  Recall@10, Effective Validation Recall@10, or broad context-area recall across
  the four-repo proof.
- ctxpack remains File Recall@10 `0.47460318`, Source Recall@10 `0.7166667`,
  and broad context-area recall `1.0`.
- VeriSchema remains File Recall@10 `0.18449473`, Source Recall@10
  `0.31067252`, Test Recall@10 `0.7089947`, Effective Validation Recall@10
  `1.0`, and broad context-area recall `0.71851856`.

## Notes

This phase intentionally improves the progressive read surface, not top-10
ranking. It gives agents a better next action for `area_context_only` gaps while
preserving the measured retrieval channels.
