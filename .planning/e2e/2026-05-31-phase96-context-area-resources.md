# Phase 96: Context Area MCP Resources

## Goal

Make broad-task area hints consumable through MCP resources, not only pack
prose. Phase 95 told agents which zero-selected areas to inspect next; Phase 96
adds source-free area resource URIs so those agents can read bounded area
details through MCP without expanding the six-tool surface or adding source
text.

## Implementation

- Added `resourceUri` to `ContextArea` as an additive source-free contract
  field.
- Moved context-area path grouping and URI encoding into `ctxpack-core` so
  compiler, eval, pack rendering, previews, and MCP use the same area names.
- Added static MCP resource `ctxpack://repo/context-areas` for a source-free
  repository area inventory.
- Added dynamic MCP resources of the form
  `ctxpack://repo/context-area/{encoded-area}` for source-free representative
  paths and role counts inside one area.
- Added context-area resources to agent previews and generated pack guidance.
- Kept the MCP tool surface unchanged at six tools.

## Evidence

Focused tests:

```bash
cargo test -p ctxpack-core context_area_resource_uri_round_trips_source_free_area_names -- --nocapture
cargo test -p ctxpack-compiler compile_context_pack_renders_context_areas -- --nocapture
cargo test -p ctxpack-mcp resources_public_uri_shapes_are_stable -- --nocapture
```

Broad proof:

```bash
cargo run --release -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-31-phase92-area-aware-gap-proof-config.json \
  --format json > .ctxpack/e2e/phase96-context-area-resources-proof.json

python3 scripts/check-product-proof.py \
  .ctxpack/e2e/phase96-context-area-resources-proof.json
```

Committed proof:

- `.ctxpack/e2e/phase96-context-area-resources-proof.json`

Result:

- `releaseGate.decision = promote`
- RefactoringMiner cold runtime is `3862ms`
- VeriSchema broad context-area recall remains `0.71851856`
- VeriSchema File Recall@10 remains `0.18449473`
- VeriSchema Source Recall@10 remains `0.31067252`
- VeriSchema Test Recall@10 remains `0.7089947`
- VeriSchema Effective Validation Recall@10 remains `1.0`
- VeriSchema protected target miss-rate remains `0.2857143`

## Notes

This phase improves progressive agent consumption rather than target-file
recall. The new resources expose only paths, role counts, diagnostics, and
privacy metadata. Agents still use their native read tools for source text.
